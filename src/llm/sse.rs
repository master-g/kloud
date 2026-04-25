//! SSE (Server-Sent Events) frame decoder.
//!
//! SSE events are delimited by double newlines (`\n\n`). This module provides
//! a [`tokio_util::codec::Decoder`] that buffers incoming bytes and yields
//! complete SSE frames as [`String`]s, solving the problem of events being
//! split across TCP chunks.

use tokio_util::codec::Decoder;

/// Splits a byte stream into SSE frames delimited by `\n\n`.
///
/// Each yielded [`String`] contains the raw text of one SSE frame
/// (without the trailing `\n\n`). The caller is responsible for
/// parsing `data:` / `event:` prefixes from the frame text.
pub struct SseDecoder;

impl Decoder for SseDecoder {
    type Item = String;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(pos) = src.windows(2).position(|w| w == b"\n\n") {
            // Consume the frame and the `\n\n` delimiter.
            let frame = src.split_to(pos + 2);
            let text = String::from_utf8_lossy(&frame[..pos]).to_string();
            Ok(Some(text))
        } else {
            // Not enough data yet — wait for more bytes.
            Ok(None)
        }
    }

    // `decode_eof` default calls `decode` once more. If the connection closes
    // with a partial frame (no trailing `\n\n`), that data is intentionally
    // discarded — an incomplete SSE frame is not actionable.
}
