use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use futures::future::Either;
use opcua_core::comms::sequence_number::SequenceNumberHandle;
use opcua_core::{trace_read_lock, trace_write_lock, RequestMessage, ResponseMessage};
use parking_lot::RwLock;
use tracing::{debug, error, trace, warn};

use opcua_core::comms::buffer::SendBuffer;
use opcua_core::comms::message_chunk::MessageIsFinalType;
use opcua_core::comms::{
    chunker::Chunker, message_chunk::MessageChunk, message_chunk_info::ChunkInfo,
    secure_channel::SecureChannel, tcp_codec::Message,
};
use opcua_types::{Error, StatusCode};

#[derive(Debug)]
struct MessageChunkWithChunkInfo {
    header: ChunkInfo,
    data_with_header: Vec<u8>,
}

pub(crate) struct MessageState {
    callback: tokio::sync::oneshot::Sender<Result<ResponseMessage, StatusCode>>,
    chunks: Vec<MessageChunkWithChunkInfo>,
    deadline: Instant,
}

pub(super) struct TransportState {
    /// Channel for outgoing requests. Will only be polled if the number of inflight requests is below the limit.
    outgoing_recv: tokio::sync::mpsc::Receiver<OutgoingMessage>,
    /// State of pending requests
    message_states: HashMap<u32, MessageState>,
    /// Secure channel
    pub(super) secure_channel: Arc<RwLock<SecureChannel>>,
    /// Max pending incoming messages
    max_pending_incoming: usize,
    /// Last decoded sequence number
    sequence_numbers: SequenceNumberHandle,
    /// Max size of incoming chunks
    #[allow(unused)]
    receive_buffer_size: usize,
}

#[derive(Debug)]
/// Result of polling a transport implementation.
/// This represents a single iteration of the transport event loop.
pub enum TransportPollResult {
    /// An outgoing message was received and enqueued.
    OutgoingMessage,
    /// An outgoing message was sent to the server.
    OutgoingMessageSent,
    /// An incoming message was received from the server.
    IncomingMessage,
    /// An error occured that is recoverable, so the transport can continue and
    /// simply fail the request.
    RecoverableError(StatusCode),
    /// The transport was closed with the given status code.
    Closed(StatusCode),
}

pub struct OutgoingMessage {
    pub request: RequestMessage,
    pub callback: Option<tokio::sync::oneshot::Sender<Result<ResponseMessage, StatusCode>>>,
    pub deadline: Instant,
}

impl TransportState {
    pub(super) fn new(
        secure_channel: Arc<RwLock<SecureChannel>>,
        outgoing_recv: tokio::sync::mpsc::Receiver<OutgoingMessage>,
        max_pending_incoming: usize,
        receive_buffer_size: usize,
    ) -> Self {
        let legacy_sequence_numbers = secure_channel
            .read()
            .security_policy()
            .legacy_sequence_numbers();
        Self {
            secure_channel,
            outgoing_recv,
            message_states: HashMap::new(),
            sequence_numbers: SequenceNumberHandle::new(legacy_sequence_numbers),
            max_pending_incoming,
            receive_buffer_size,
        }
    }

    /// Wait for an outgoing message. Will also check for timed out messages.
    pub(super) async fn wait_for_outgoing_message(
        &mut self,
        send_buffer: &mut SendBuffer,
    ) -> Option<(RequestMessage, u32)> {
        loop {
            // Check for any messages that have timed out, and get the time until the next message
            // times out
            let timeout_fut = match self.next_timeout() {
                Some(t) => Either::Left(tokio::time::sleep_until(t.into())),
                None => Either::Right(futures::future::pending::<()>()),
            };

            tokio::select! {
                    _ = timeout_fut => {
                        continue;
                    }
                    outgoing = self.outgoing_recv.recv() => {
                        let outgoing = outgoing?;
                        let request_id = send_buffer.next_request_id();
                        if let Some(callback) = outgoing.callback {
                            self.message_states.insert(request_id, MessageState {
                                callback,
                                chunks: Vec::new(),
                                deadline: outgoing.deadline,
                            });
                        }
                        break Some((outgoing.request, request_id));
                    }
            }
        }
    }

    /// Store incoming messages in the message state.
    pub(super) fn handle_incoming_message(&mut self, message: Message) -> Result<(), StatusCode> {
        let status = match message {
            Message::Acknowledge(ack) => {
                debug!("Reader got an unexpected ack {:?}", ack);
                StatusCode::BadUnexpectedError
            }
            Message::Chunk(chunk) => self.process_chunk(chunk).err().unwrap_or(StatusCode::Good),
            Message::Error(error) => {
                error!(
                    "Received error {} from server. Reason: {}",
                    error.error, error.reason
                );
                error.error
            }
            m => {
                error!("Expected a recognized message, got {:?}", m);
                StatusCode::BadUnexpectedError
            }
        };

        if status.is_good() {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub(super) fn message_send_failed(&mut self, request_id: u32, err: StatusCode) {
        if let Some(message_state) = self.message_states.remove(&request_id) {
            let _ = message_state.callback.send(Err(err));
        }
    }

    fn next_timeout(&mut self) -> Option<Instant> {
        let now = Instant::now();
        let mut next_timeout = None;
        let mut timed_out = Vec::new();
        for (id, state) in &self.message_states {
            if state.deadline <= now {
                timed_out.push(*id);
            } else {
                match &next_timeout {
                    Some(t) if *t > state.deadline => next_timeout = Some(state.deadline),
                    None => next_timeout = Some(state.deadline),
                    _ => {}
                }
            }
        }
        for id in timed_out {
            if let Some(state) = self.message_states.remove(&id) {
                debug!("Message {} timed out", id);
                let _ = state.callback.send(Err(StatusCode::BadTimeout));
            }
        }
        next_timeout
    }

    fn process_chunk(&mut self, chunk: MessageChunk) -> Result<(), StatusCode> {
        let mut secure_channel = trace_write_lock!(self.secure_channel);
        let chunk = secure_channel.verify_and_remove_security(&chunk.data)?;

        let chunk_info = chunk.chunk_info(&secure_channel)?;
        drop(secure_channel);
        let req_id = chunk_info.sequence_header.request_id;

        // We do not care at all about incoming messages without a
        // corresponding request.
        let Some(message_state) = self.message_states.get_mut(&req_id) else {
            return Ok(());
        };

        match chunk_info.message_header.is_final {
            MessageIsFinalType::Intermediate => {
                trace!(
                    "receive chunk intermediate {}:{}. Length {}",
                    chunk_info.sequence_header.request_id,
                    chunk_info.sequence_header.sequence_number,
                    chunk_info.body_length
                );
                message_state.chunks.push(MessageChunkWithChunkInfo {
                    header: chunk_info,
                    data_with_header: chunk.data,
                });
                let chunks_len: usize = message_state.chunks.len();
                if self.max_pending_incoming > 0 && chunks_len > self.max_pending_incoming {
                    error!(
                        "too many pending incoming messages {} > {}",
                        chunks_len, self.max_pending_incoming
                    );
                    let message_state = self.message_states.remove(&req_id).unwrap();
                    let _ = message_state
                        .callback
                        .send(Err(StatusCode::BadEncodingLimitsExceeded));
                }
            }
            MessageIsFinalType::FinalError => {
                warn!("Discarding chunk marked in as final error");
                let message_state = self.message_states.remove(&req_id).unwrap();
                let _ = message_state
                    .callback
                    .send(Err(StatusCode::BadCommunicationError));
            }
            MessageIsFinalType::Final => {
                trace!(
                    "receive chunk final {}:{}. Length {}",
                    chunk_info.sequence_header.request_id,
                    chunk_info.sequence_header.sequence_number,
                    chunk_info.body_length
                );
                message_state.chunks.push(MessageChunkWithChunkInfo {
                    header: chunk_info,
                    data_with_header: chunk.data,
                });
                let message_state = self.message_states.remove(&req_id).unwrap();
                let in_chunks = Self::merge_chunks(message_state.chunks)?;
                let message = self.turn_received_chunks_into_message(&in_chunks)?;

                let _ = message_state.callback.send(Ok(message));
            }
        }
        Ok(())
    }

    fn turn_received_chunks_into_message(
        &mut self,
        chunks: &[MessageChunk],
    ) -> Result<ResponseMessage, Error> {
        // Validate that all chunks have incrementing sequence numbers and valid chunk types
        let secure_channel = trace_read_lock!(self.secure_channel);
        self.sequence_numbers.set(Chunker::validate_chunks(
            self.sequence_numbers.clone(),
            &secure_channel,
            chunks,
        )?);
        // Now decode
        Chunker::decode(chunks, &secure_channel, None)
    }

    fn merge_chunks(
        mut chunks: Vec<MessageChunkWithChunkInfo>,
    ) -> Result<Vec<MessageChunk>, StatusCode> {
        if chunks.len() == 1 {
            return Ok(vec![MessageChunk {
                data: chunks.pop().unwrap().data_with_header,
            }]);
        }
        chunks.sort_by(|a, b| {
            a.header
                .sequence_header
                .sequence_number
                .cmp(&b.header.sequence_header.sequence_number)
        });
        let mut ret = Vec::with_capacity(chunks.len());
        let mut expect_sequence_number = chunks
            .first()
            .unwrap()
            .header
            .sequence_header
            .sequence_number;
        for c in chunks {
            if c.header.sequence_header.sequence_number != expect_sequence_number {
                warn!(
                    "receive wrong chunk expect seq={} got={}",
                    expect_sequence_number, c.header.sequence_header.sequence_number
                );
                continue; //may be duplicate chunk
            }
            expect_sequence_number += 1;
            ret.push(MessageChunk {
                data: c.data_with_header,
            });
        }
        Ok(ret)
    }

    /// Close the transport, aborting any pending requests.
    /// If `status` is good, the pending requests will be terminated with
    /// `BadConnectionClosed`.
    pub(super) async fn close(&mut self, status: StatusCode) -> StatusCode {
        // If the status is good, we still want to send a bad status code
        // to the pending requests. They didn't succeed, after all.
        let request_status = if status.is_good() {
            StatusCode::BadConnectionClosed
        } else {
            status
        };

        for (_, pending) in self.message_states.drain() {
            let _ = pending.callback.send(Err(request_status));
        }

        // Make sure we also send a bad status for any remaining messages in the queue
        // Close the channel first.
        self.outgoing_recv.close();

        // recv is no longer blocking.
        while let Some(msg) = self.outgoing_recv.recv().await {
            if let Some(cb) = msg.callback {
                let _ = cb.send(Err(request_status));
            }
        }

        status
    }
}
