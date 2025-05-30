//! Shared implementaton of an OPC-UA buffer, handling
//! encoding of data and the state of a communication channel.

use std::{
    collections::VecDeque,
    io::{BufRead, Cursor},
};

use tokio::io::AsyncWriteExt;
use tracing::trace;

use crate::{
    comms::{chunker::Chunker, message_chunk::MessageChunk, secure_channel::SecureChannel},
    Message,
};

use opcua_types::{Error, SimpleBinaryEncodable, StatusCode};

use super::{
    sequence_number::SequenceNumberHandle,
    tcp_types::{AcknowledgeMessage, ErrorMessage},
};

#[derive(Copy, Clone, Debug)]
enum SendBufferState {
    Reading(usize),
    Writing,
}

#[derive(Debug)]
enum PendingPayload {
    Chunk(MessageChunk),
    Ack(AcknowledgeMessage),
    Error(ErrorMessage),
}

/// General implementation of a buffer of outgoing messages.
pub struct SendBuffer {
    /// The send buffer
    buffer: Cursor<Vec<u8>>,
    /// Queued chunks
    chunks: VecDeque<PendingPayload>,
    /// The last request id
    last_request_id: u32,
    /// Last sent sequence number
    sequence_numbers: SequenceNumberHandle,
    /// Maximum size of a message, total. Use 0 for no limit
    pub max_message_size: usize,
    /// Maximum number of chunks in a message.
    pub max_chunk_count: usize,
    /// Maximum size of each individual chunk.
    pub send_buffer_size: usize,

    state: SendBufferState,
}

// The send buffer works as follows:
//  - `write` is called with a message that is written to the internal buffer.
//  - `read_into_async` is called, which sets the state to `Writing`.
//  - Once the buffer is exhausted, the state is set back to `Reading`.
//  - `write` cannot be called while we are writing to the output.
impl SendBuffer {
    /// Create a new send buffer with the given initial limits.
    pub fn new(
        buffer_size: usize,
        max_message_size: usize,
        max_chunk_count: usize,
        sequence_numbers_legacy: bool,
    ) -> Self {
        Self {
            buffer: Cursor::new(vec![0u8; buffer_size + 1024]),
            chunks: VecDeque::with_capacity(max_chunk_count),
            last_request_id: 1000,
            sequence_numbers: SequenceNumberHandle::new(sequence_numbers_legacy),
            max_message_size,
            max_chunk_count,
            send_buffer_size: buffer_size,
            state: SendBufferState::Writing,
        }
    }

    /// Encode the next chunk in the queue to the out-buffer.
    pub fn encode_next_chunk(&mut self, secure_channel: &SecureChannel) -> Result<(), StatusCode> {
        if matches!(self.state, SendBufferState::Reading(_)) {
            return Err(StatusCode::BadInvalidState);
        }

        let Some(next_chunk) = self.chunks.pop_front() else {
            return Ok(());
        };

        let size = match next_chunk {
            PendingPayload::Chunk(c) => secure_channel.apply_security(&c, self.buffer.get_mut())?,
            PendingPayload::Ack(a) => {
                a.encode(&mut self.buffer)?;
                self.buffer.position() as usize
            }
            PendingPayload::Error(e) => {
                e.encode(&mut self.buffer)?;
                self.buffer.position() as usize
            }
        };
        self.buffer.set_position(0);
        self.state = SendBufferState::Reading(size);

        Ok(())
    }

    /// Set whether we are using legacy sequence numbers or not.
    /// This depends on the active security policy.
    pub fn set_sequence_number_legacy(&mut self, is_legacy: bool) {
        self.sequence_numbers.set_is_legacy(is_legacy);
    }

    /// Clear the list of pending messages, then
    /// add an error.
    pub fn write_error(&mut self, error: ErrorMessage) {
        // Clear any pending chunks, we're erroring out
        self.chunks.clear();
        self.chunks.push_back(PendingPayload::Error(error));
    }

    /// Write an acknowledge message to the list of pending messages.
    pub fn write_ack(&mut self, ack: AcknowledgeMessage) {
        self.chunks.push_back(PendingPayload::Ack(ack));
    }

    /// Encode a message to chunks, then write them to the pending message queue.
    ///
    /// The messages are encrypted as they are sent.
    pub fn write(
        &mut self,
        request_id: u32,
        message: impl Message,
        secure_channel: &SecureChannel,
    ) -> Result<u32, Error> {
        trace!("Writing request to buffer");

        // Turn message to chunk(s)
        let chunks = Chunker::encode(
            self.sequence_numbers.clone(),
            request_id,
            self.max_message_size,
            self.send_buffer_size,
            secure_channel,
            &message,
        )
        .map_err(|e| e.with_context(Some(request_id), Some(message.request_handle())))?;

        if self.max_chunk_count > 0 && chunks.len() > self.max_chunk_count {
            Err(Error::new(
                StatusCode::BadCommunicationError,
                format!(
                    "Cannot write message since {} chunks exceeds {} chunk limit",
                    chunks.len(),
                    self.max_chunk_count
                ),
            )
            .with_context(Some(request_id), Some(message.request_handle())))
        } else {
            // Sequence number monotonically increases per chunk
            self.sequence_numbers.increment(chunks.len() as u32);

            // Send chunks
            self.chunks
                .extend(chunks.into_iter().map(PendingPayload::Chunk));
            Ok(request_id)
        }
    }

    /// Get the next request ID.
    pub fn next_request_id(&mut self) -> u32 {
        self.last_request_id += 1;
        self.last_request_id
    }

    /// Read the pending buffer into the given stream.
    pub async fn read_into_async(
        &mut self,
        write: &mut (impl tokio::io::AsyncWrite + Unpin),
    ) -> Result<(), tokio::io::Error> {
        // Set the state to writing, or get the current end point
        let end = match self.state {
            SendBufferState::Writing => {
                let end = self.buffer.position() as usize;
                self.state = SendBufferState::Reading(end);
                self.buffer.set_position(0);
                end
            }
            SendBufferState::Reading(end) => end,
        };

        let pos = self.buffer.position() as usize;
        let buf = &self.buffer.get_ref()[pos..end];
        // Write to the stream, note that we do not actually advance the stream before
        // after we have written. This means that since `write` is cancellation safe, our stream is
        // cancellation safe, which is essential.
        let written = write.write(buf).await?;

        self.buffer.consume(written);

        if end == self.buffer.position() as usize {
            self.state = SendBufferState::Writing;
            self.buffer.set_position(0);
        }

        Ok(())
    }

    /// Return `true` if we should encode a new chunk.
    pub fn should_encode_chunks(&self) -> bool {
        !self.chunks.is_empty() && !self.can_read()
    }

    /// Check if we can read data from the buffer into the stream.
    pub fn can_read(&self) -> bool {
        matches!(self.state, SendBufferState::Reading(_)) || self.buffer.position() != 0
    }

    /// Revise the limits with the result of a hello/acknowledge message.
    pub fn revise(
        &mut self,
        send_buffer_size: usize,
        max_message_size: usize,
        max_chunk_count: usize,
    ) {
        if self.send_buffer_size > send_buffer_size {
            self.buffer.get_mut().shrink_to(send_buffer_size + 1024);
            self.send_buffer_size = send_buffer_size;
        }
        if self.max_message_size > max_message_size && max_message_size > 0 {
            self.max_message_size = max_message_size;
        }
        if self.max_chunk_count > max_chunk_count && max_chunk_count > 0 {
            self.max_chunk_count = max_chunk_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;

    use parking_lot::RwLock;

    use super::SendBuffer;

    use crate::comms::secure_channel::{Role, SecureChannel};
    use crate::RequestMessage;
    use opcua_crypto::CertificateStore;
    use opcua_types::StatusCode;
    use opcua_types::{
        DateTime, NodeId, ReadRequest, ReadValueId, RequestHeader, TimestampsToReturn,
    };

    fn get_buffer_and_channel() -> (SendBuffer, SecureChannel) {
        let buffer = SendBuffer::new(8196, 81960, 5, true);
        let channel = SecureChannel::new(
            Arc::new(RwLock::new(CertificateStore::new(std::path::Path::new(
                "./pki",
            )))),
            Role::Client,
            Default::default(),
        );

        (buffer, channel)
    }

    #[tokio::test]
    async fn test_buffer_simple() {
        // Write a small message to the buffer
        let message = ReadRequest {
            request_header: RequestHeader::new(&NodeId::null(), &DateTime::null(), 101),
            max_age: 0.0,
            timestamps_to_return: TimestampsToReturn::Both,
            nodes_to_read: Some(vec![ReadValueId {
                node_id: (1, 1).into(),
                attribute_id: 1,
                ..Default::default()
            }]),
        };

        let (mut buffer, channel) = get_buffer_and_channel();

        let m: RequestMessage = message.into();
        let request_id = buffer.write(1, m, &channel).unwrap();
        assert_eq!(request_id, 1);

        assert!(buffer.should_encode_chunks());
        assert_eq!(buffer.chunks.len(), 1);
        buffer.encode_next_chunk(&channel).unwrap();
        assert!(buffer.can_read());

        let mut cursor = Cursor::new(Vec::new());
        buffer.read_into_async(&mut cursor).await.unwrap();
        assert!(cursor.get_ref().len() > 50);
    }

    #[tokio::test]
    async fn test_buffer_chunking() {
        // Write a large enough message that it is split into chunks.
        let message = ReadRequest {
            request_header: RequestHeader::new(&NodeId::null(), &DateTime::null(), 101),
            max_age: 0.0,
            timestamps_to_return: TimestampsToReturn::Both,
            nodes_to_read: Some(
                (0..1000)
                    .map(|r| ReadValueId {
                        node_id: (1, r).into(),
                        attribute_id: 1,
                        ..Default::default()
                    })
                    .collect(),
            ),
        };

        let (mut buffer, channel) = get_buffer_and_channel();

        let m: RequestMessage = message.into();
        let request_id = buffer.write(1, m, &channel).unwrap();
        assert_eq!(request_id, 1);

        assert_eq!(buffer.chunks.len(), 3);
        let mut cursor = Cursor::new(Vec::new());

        for _ in 0..3 {
            assert!(buffer.should_encode_chunks());
            buffer.encode_next_chunk(&channel).unwrap();
            assert!(!buffer.should_encode_chunks());
            assert!(buffer.can_read());

            buffer.read_into_async(&mut cursor).await.unwrap();
        }
        assert!(!buffer.should_encode_chunks());
        assert!(!buffer.can_read());
        assert!(cursor.get_ref().len() > 8196 * 2 && cursor.get_ref().len() < 8196 * 3);
    }

    #[test]
    fn test_buffer_too_large_message() {
        // Write a very large message exceeding the max message size.
        let message = ReadRequest {
            request_header: RequestHeader::new(&NodeId::null(), &DateTime::null(), 101),
            max_age: 0.0,
            timestamps_to_return: TimestampsToReturn::Both,
            nodes_to_read: Some(
                (0..10000)
                    .map(|r| ReadValueId {
                        node_id: (1, r).into(),
                        attribute_id: 1,
                        ..Default::default()
                    })
                    .collect(),
            ),
        };

        let (mut buffer, channel) = get_buffer_and_channel();

        let m: RequestMessage = message.into();
        let err = buffer.write(1, m, &channel).unwrap_err();
        assert_eq!(err.status(), StatusCode::BadRequestTooLarge);
    }

    #[test]
    fn test_buffer_too_many_chunks() {
        // Write a large enough message that we exceed the maximum chunk count.
        let message = ReadRequest {
            request_header: RequestHeader::new(&NodeId::null(), &DateTime::null(), 101),
            max_age: 0.0,
            timestamps_to_return: TimestampsToReturn::Both,
            nodes_to_read: Some(
                (0..4000)
                    .map(|r| ReadValueId {
                        node_id: (1, r).into(),
                        attribute_id: 1,
                        ..Default::default()
                    })
                    .collect(),
            ),
        };

        let (mut buffer, channel) = get_buffer_and_channel();

        let m: RequestMessage = message.into();
        let err = buffer.write(1, m, &channel).unwrap_err();
        assert_eq!(err.status(), StatusCode::BadCommunicationError);
    }

    #[tokio::test]
    async fn test_buffer_read_partial() {
        // Write a large message to the buffer.
        let message = ReadRequest {
            request_header: RequestHeader::new(&NodeId::null(), &DateTime::null(), 101),
            max_age: 0.0,
            timestamps_to_return: TimestampsToReturn::Both,
            nodes_to_read: Some(
                (0..1000)
                    .map(|r| ReadValueId {
                        node_id: (1, r).into(),
                        attribute_id: 1,
                        ..Default::default()
                    })
                    .collect(),
            ),
        };

        let (mut buffer, channel) = get_buffer_and_channel();

        let m: RequestMessage = message.into();
        let request_id = buffer.write(1, m, &channel).unwrap();
        assert_eq!(request_id, 1);

        assert_eq!(buffer.chunks.len(), 3);
        // Use a fixed size buffer exactly half the chunk size. This simulates a TCP connection
        // writing data in smaller chunks than configured chunk size.
        let mut buf = [0u8; 4098];
        // Cursor<&mut [u8; N]> doesn't support AsyncWrite, but Cursor<&mut [u8]> does.
        let mut cursor = Cursor::new(&mut buf as &mut [u8]);

        for _ in 0..2 {
            println!("Encode chunks");
            assert!(buffer.should_encode_chunks());
            buffer.encode_next_chunk(&channel).unwrap();
            assert!(!buffer.should_encode_chunks());
            assert!(buffer.can_read());

            buffer.read_into_async(&mut cursor).await.unwrap();
            assert!(buffer.can_read());
            assert_eq!(cursor.position(), 4098);
            cursor.set_position(0);
            buffer.read_into_async(&mut cursor).await.unwrap();
            assert!(!buffer.can_read());
            assert_eq!(cursor.position(), 4098);
            cursor.set_position(0);
        }
        assert!(buffer.should_encode_chunks());
        buffer.encode_next_chunk(&channel).unwrap();
        assert!(buffer.can_read());
        buffer.read_into_async(&mut cursor).await.unwrap();
        assert!(cursor.position() < 4098);

        assert!(!buffer.should_encode_chunks());
        assert!(!buffer.can_read());
    }
}
