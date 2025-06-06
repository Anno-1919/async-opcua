use std::{sync::Arc, time::Instant};

use futures::{future::Either, stream::FuturesUnordered, Future, Stream, StreamExt};
use opcua_types::StatusCode;
use tracing::debug;

use crate::{
    session::{session_debug, session_error},
    Session,
};

/// An event on the subscription event loop.
#[derive(Debug)]
pub enum SubscriptionActivity {
    /// A publish request received a successful response.
    Publish,
    /// A publish request failed, either due to a timeout or an error.
    /// The publish request will typically be retried.
    PublishFailed(StatusCode),
}

/// An event loop for running periodic subscription tasks.
///
/// This handles publshing on a fixed interval, republishing failed requests,
/// and subscription keep-alive.
pub(crate) struct SubscriptionEventLoop {
    session: Arc<Session>,
    trigger_publish_recv: tokio::sync::watch::Receiver<Instant>,
    last_external_trigger: Instant,
    // This is true if the client has received BadTooManyPublishRequests
    // and is waiting for a response before making further requests.
    waiting_for_response: bool,
    // This is true if the client has received a no_subscriptions response,
    // and is waiting for a manual trigger or successful response before resuming publishing.
    no_active_subscription: bool,
}

impl SubscriptionEventLoop {
    /// Create a new subscription event loop for `session`
    ///
    /// # Arguments
    ///
    ///  * `session` - A shared reference to an [AsyncSession].
    ///  * `trigger_publish_recv` - A channel used to transmit external publish triggers.
    ///    This is used to trigger publish outside of the normal schedule, for example when
    ///    a new subscription is created.
    pub(crate) fn new(
        session: Arc<Session>,
        trigger_publish_recv: tokio::sync::watch::Receiver<Instant>,
    ) -> Self {
        let last_external_trigger = *trigger_publish_recv.borrow();
        Self {
            last_external_trigger,
            trigger_publish_recv,
            session,
            waiting_for_response: false,
            no_active_subscription: false,
        }
    }

    /// Run the subscription event loop, returning a stream that produces
    /// [SubscriptionActivity] enums, reporting activity to the session event loop.
    pub(crate) fn run(self) -> impl Stream<Item = SubscriptionActivity> {
        futures::stream::unfold(
            (self, FuturesUnordered::new()),
            |(mut slf, mut futures)| async move {
                // Store the next publish time, or None if there are no active subscriptions.
                let mut next = slf.session.next_publish_time(false);
                let mut recv: tokio::sync::watch::Receiver<Instant> =
                    slf.trigger_publish_recv.clone();

                let res = loop {
                    // Future for the next periodic publish. We do not send publish requests
                    // if there are no active subscriptions. In this case, simply return the
                    // non-terminating future.
                    let next_tick_fut = if let Some(next) = next {
                        if slf.waiting_for_response && !futures.is_empty() {
                            Either::Right(futures::future::pending::<()>())
                        } else {
                            Either::Left(tokio::time::sleep_until(next.into()))
                        }
                    } else {
                        Either::Right(futures::future::pending::<()>())
                    };

                    // If FuturesUnordered is empty, it will immediately yield `None`. We don't
                    // want that, so if it is empty we return the non-terminating future.
                    let next_publish_fut = if futures.is_empty() {
                        Either::Left(futures::future::pending())
                    } else {
                        Either::Right(futures.next())
                    };

                    tokio::select! {
                        // Both internal ticks and external triggers result in publish requests.
                        v = recv.wait_for(|i| i > &slf.last_external_trigger) => {
                            if let Ok(v) = v {
                                if !slf.waiting_for_response {
                                    debug!("Sending publish due to external trigger");
                                    // On an external trigger, we always publish.
                                    futures.push(slf.static_publish());
                                    next = slf.session.next_publish_time(true);
                                    slf.last_external_trigger = *v;
                                } else {
                                    debug!("Skipping publish due BadTooManyPublishRequests");
                                }
                            }
                            slf.no_active_subscription = false;
                        }
                        _ = next_tick_fut => {
                            // Avoid publishing if there are too many inflight publish requests.
                            if !slf.no_active_subscription && futures.len()
                                < slf
                                    .session
                                    .publish_limits_watch_rx
                                    .borrow()
                                    .max_publish_requests
                            {
                                if !slf.waiting_for_response {
                                    debug!("Sending publish due to internal tick");
                                    futures.push(slf.static_publish());
                                } else {
                                    debug!("Skipping publish due BadTooManyPublishRequests");
                                }
                            }
                            next = slf.session.next_publish_time(true);
                        }
                        res = next_publish_fut => {
                            match res {
                                Some(Ok(more_notifications)) => {
                                    if more_notifications
                                        || futures.len()
                                            < slf
                                                .session
                                                .publish_limits_watch_rx
                                                .borrow()
                                                .min_publish_requests
                                    {
                                        if !slf.waiting_for_response {
                                            debug!("Sending publish after receiving response");
                                            futures.push(slf.static_publish());
                                            // Set the last publish time to to avoid a buildup
                                            // of publish requests if exhausting the queue takes
                                            // more time than a single publishing interval.
                                            slf.session.next_publish_time(true);
                                        } else {
                                            debug!("Skipping publish due BadTooManyPublishRequests");
                                        }
                                    }
                                    slf.waiting_for_response = false;
                                    slf.no_active_subscription = false;
                                    break SubscriptionActivity::Publish
                                }
                                Some(Err(e)) => {
                                    match e {
                                        StatusCode::BadTimeout => {
                                            session_debug!(slf.session, "Publish request timed out");
                                        }
                                        StatusCode::BadTooManyPublishRequests => {
                                            session_debug!(
                                                slf.session,
                                                "Server returned BadTooManyPublishRequests, backing off",
                                            );
                                            slf.waiting_for_response = true;
                                        }
                                        StatusCode::BadSessionClosed
                                        | StatusCode::BadSessionIdInvalid => {
                                            // If this happens we will probably eventually fail keep-alive, defer to that.
                                            session_error!(slf.session, "Publish response indicates session is dead");
                                        }
                                        StatusCode::BadNoSubscription => {
                                            session_debug!(
                                                slf.session,
                                                "Publish response indicates that there are no subscriptions"
                                            );
                                            slf.no_active_subscription = true;
                                        },
                                        _ => ()
                                    }
                                    break SubscriptionActivity::PublishFailed(e)
                                }
                                // Should be impossible.
                                None => break SubscriptionActivity::PublishFailed(
                                    StatusCode::BadInvalidState,
                                )
                            }
                        }
                    }
                };

                Some((res, (slf, futures)))
            },
        )
    }

    fn static_publish(&self) -> impl Future<Output = Result<bool, StatusCode>> + 'static {
        let inner_session = self.session.clone();
        async move { inner_session.publish().await }
    }
}
