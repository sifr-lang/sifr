use crate::{IPC_LENGTH_PREFIX_BYTES, IpcEnvelope, IpcFrameError, decode_frame, encode_frame};
use std::io::{Read, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcTransportError {
    Frame(IpcFrameError),
    Read,
    Write,
}

impl From<IpcFrameError> for IpcTransportError {
    fn from(error: IpcFrameError) -> Self {
        Self::Frame(error)
    }
}

pub fn read_frame<R: Read>(
    reader: &mut R,
    max_frame_bytes: u32,
) -> Result<Option<IpcEnvelope>, IpcTransportError> {
    let mut prefix = [0_u8; IPC_LENGTH_PREFIX_BYTES];
    let Some(prefix_len) = read_prefix(reader, &mut prefix)? else {
        return Ok(None);
    };
    if prefix_len < IPC_LENGTH_PREFIX_BYTES {
        return Err(IpcFrameError::LengthPrefixTruncated {
            received: prefix_len,
        }
        .into());
    }

    let frame_len = u32::from_le_bytes(prefix);
    if frame_len > max_frame_bytes {
        return Err(IpcFrameError::FrameTooLarge {
            frame_len,
            max_frame_bytes,
        }
        .into());
    }
    let payload_len = usize::try_from(frame_len).map_err(|_| {
        IpcTransportError::Frame(IpcFrameError::LengthUnsupported {
            frame_len: usize::MAX,
        })
    })?;
    let payload = read_payload(reader, frame_len, payload_len)?;

    let mut frame = Vec::with_capacity(IPC_LENGTH_PREFIX_BYTES + payload.len());
    frame.extend_from_slice(&prefix);
    frame.extend_from_slice(&payload);
    decode_frame(&frame, max_frame_bytes)
        .map(Some)
        .map_err(IpcTransportError::Frame)
}

pub fn write_frame<W: Write>(
    writer: &mut W,
    envelope: &IpcEnvelope,
    max_frame_bytes: u32,
) -> Result<(), IpcTransportError> {
    let encoded = encode_frame(envelope, max_frame_bytes)?;
    writer
        .write_all(&encoded)
        .map_err(|_| IpcTransportError::Write)?;
    writer.flush().map_err(|_| IpcTransportError::Write)
}

fn read_prefix<R: Read>(
    reader: &mut R,
    prefix: &mut [u8; IPC_LENGTH_PREFIX_BYTES],
) -> Result<Option<usize>, IpcTransportError> {
    let mut received = 0;
    while received < IPC_LENGTH_PREFIX_BYTES {
        match reader.read(&mut prefix[received..]) {
            Ok(0) if received == 0 => return Ok(None),
            Ok(0) => return Ok(Some(received)),
            Ok(count) => received += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(IpcTransportError::Read),
        }
    }
    Ok(Some(received))
}

fn read_payload<R: Read>(
    reader: &mut R,
    frame_len: u32,
    payload_len: usize,
) -> Result<Vec<u8>, IpcTransportError> {
    let mut payload = vec![0_u8; payload_len];
    let mut received = 0;
    while received < payload_len {
        match reader.read(&mut payload[received..]) {
            Ok(0) => {
                return Err(IpcTransportError::Frame(IpcFrameError::PayloadTruncated {
                    expected: frame_len,
                    received,
                }));
            }
            Ok(count) => received += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(IpcTransportError::Read),
        }
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::{IpcTransportError, read_frame, write_frame};
    use crate::{
        IPC_DEFAULT_MAX_FRAME_BYTES, IPC_LENGTH_PREFIX_BYTES, IpcEnvelope, IpcFrameError,
        IpcWireSchema,
    };
    use std::io::{Cursor, Error, ErrorKind, Write};

    fn sample_schema() -> IpcWireSchema {
        IpcWireSchema {
            name: "demo.worker.Echo".to_string(),
            version: 1,
            hash: 0x4733_c89f_b23a_40ec_b5f3_bcda_99fb_34da_u128.to_be_bytes(),
            compatible_version_min: 1,
            compatible_version_max: 1,
        }
    }

    fn sample_frame() -> IpcEnvelope {
        IpcEnvelope::Run {
            request_id: 42,
            payload: vec![1, 2, 3],
        }
    }

    #[test]
    fn write_then_read_frame_round_trips_on_pipe_shaped_stream() {
        let frame = sample_frame();
        let mut stream = Cursor::new(Vec::new());

        assert_eq!(
            write_frame(&mut stream, &frame, IPC_DEFAULT_MAX_FRAME_BYTES),
            Ok(())
        );
        stream.set_position(0);
        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Ok(Some(frame))
        );
    }

    #[test]
    fn read_frame_returns_none_for_clean_eof_before_prefix() {
        let mut stream = Cursor::new(Vec::new());

        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Ok(None)
        );
    }

    #[test]
    fn read_frame_reports_truncated_prefix_without_panicking() {
        let mut stream = Cursor::new(vec![1, 2]);

        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcTransportError::Frame(
                IpcFrameError::LengthPrefixTruncated { received: 2 }
            ))
        );
    }

    #[test]
    fn read_frame_rejects_oversize_prefix_before_payload_read() {
        let mut bytes = (IPC_DEFAULT_MAX_FRAME_BYTES + 1).to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0, 1, 2, 3]);
        let mut stream = Cursor::new(bytes);

        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcTransportError::Frame(IpcFrameError::FrameTooLarge {
                frame_len: IPC_DEFAULT_MAX_FRAME_BYTES + 1,
                max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES
            }))
        );
    }

    #[test]
    fn read_frame_reports_truncated_payload_without_panicking() {
        let mut bytes = 5_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[1, 2, 3]);
        let mut stream = Cursor::new(bytes);

        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcTransportError::Frame(IpcFrameError::PayloadTruncated {
                expected: 5,
                received: 3
            }))
        );
    }

    #[test]
    fn write_frame_reports_encode_limit_errors() {
        let mut stream = Cursor::new(Vec::new());

        assert!(matches!(
            write_frame(&mut stream, &sample_frame(), 1),
            Err(IpcTransportError::Frame(IpcFrameError::FrameTooLarge {
                frame_len: _,
                max_frame_bytes: 1
            }))
        ));
    }

    #[test]
    fn write_frame_reports_writer_errors_without_payload_text() {
        let mut writer = FailingWriter;

        assert_eq!(
            write_frame(&mut writer, &sample_frame(), IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcTransportError::Write)
        );
    }

    #[test]
    fn read_frame_accepts_bootstrap_payloads_written_by_codec() {
        let frame = IpcEnvelope::Hello {
            protocol_min: 1,
            protocol_max: 1,
            schema: sample_schema(),
            max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES,
        };
        let mut stream = Cursor::new(Vec::new());

        assert_eq!(
            write_frame(&mut stream, &frame, IPC_DEFAULT_MAX_FRAME_BYTES),
            Ok(())
        );
        stream.set_position(0);
        assert_eq!(
            read_frame(&mut stream, IPC_DEFAULT_MAX_FRAME_BYTES),
            Ok(Some(frame))
        );
    }

    #[test]
    fn length_prefix_constant_matches_stream_prefix_size() {
        assert_eq!(IPC_LENGTH_PREFIX_BYTES, 4);
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::BrokenPipe, "closed"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
