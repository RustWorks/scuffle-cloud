//! Reading [`MessageData`].

use super::{MessageData, MessageType, UnknownMessage};
use crate::chunk::Chunk;
use crate::command_messages::Command;
use crate::protocol_control_messages::{
    ProtocolControlMessageSetChunkSize, ProtocolControlMessageWindowAcknowledgementSize,
};

impl<'a> MessageData<'a> {
    /// Reads [`MessageData`] from the given chunk.
    pub fn read(chunk: &'a Chunk) -> Result<Self, crate::error::RtmpError> {
        match chunk.message_header.msg_type_id {
            // Protocol Control Messages
            MessageType::SetChunkSize => {
                let data = ProtocolControlMessageSetChunkSize::read(&chunk.payload)?;
                Ok(Self::SetChunkSize(data))
            }
            MessageType::Abort => Ok(Self::Abort), // Not implemented
            MessageType::Acknowledgement => Ok(Self::Acknowledgement), // Not implemented
            MessageType::UserControlEvent => Ok(Self::UserControlEvent), // Not implemented
            MessageType::WindowAcknowledgementSize => {
                let data = ProtocolControlMessageWindowAcknowledgementSize::read(&chunk.payload)?;
                Ok(Self::SetAcknowledgementWindowSize(data))
            }
            MessageType::SetPeerBandwidth => Ok(Self::SetPeerBandwidth), // Not implemented
            // RTMP Command Messages
            MessageType::Audio => Ok(Self::AudioData {
                data: chunk.payload.clone(),
            }),
            MessageType::Video => Ok(Self::VideoData {
                data: chunk.payload.clone(),
            }),
            MessageType::DataAMF3 => Ok(Self::DataAmf3), // Not implemented
            MessageType::SharedObjAMF3 => Ok(Self::SharedObjAmf3), // Not implemented
            MessageType::CommandAMF3 => Ok(Self::CommandAmf3), // Not implemented
            // Metadata
            MessageType::DataAMF0 => Ok(Self::DataAmf0 {
                data: chunk.payload.clone(),
            }),
            MessageType::SharedObjAMF0 => Ok(Self::SharedObjAmf0), // Not implemented
            MessageType::CommandAMF0 => Ok(Self::Amf0Command(Command::read(&chunk.payload)?)),
            MessageType::Aggregate => Ok(Self::Aggregate), // Not implemented
            msg_type_id => Ok(Self::Unknown(UnknownMessage {
                msg_type_id,
                data: chunk.payload.clone(),
            })),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;
    use scuffle_amf0::{Amf0Encoder, Amf0Value};

    use super::*;
    use crate::command_messages::CommandType;
    use crate::command_messages::netconnection::NetConnectionCommand;

    #[test]
    fn test_parse_command() {
        let mut amf0_writer = Vec::new();

        Amf0Encoder::encode_string(&mut amf0_writer, "connect").unwrap();
        Amf0Encoder::encode_number(&mut amf0_writer, 1.0).unwrap();
        Amf0Encoder::encode_object(&mut amf0_writer, &[("app".into(), Amf0Value::String("testapp".into()))]).unwrap();

        let amf_data = Bytes::from(amf0_writer);

        let chunk = Chunk::new(0, 0, MessageType::CommandAMF0, 0, amf_data);

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::Amf0Command(command) => {
                let Command {
                    transaction_id,
                    command_type,
                } = command;
                assert_eq!(transaction_id, 1.0);

                let CommandType::NetConnection(NetConnectionCommand::Connect(connect)) = command_type else {
                    panic!("wrong command");
                };

                assert_eq!(connect.app, "testapp");
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_audio_packet() {
        let chunk = Chunk::new(0, 0, MessageType::Audio, 0, vec![0x00, 0x00, 0x00, 0x00].into());

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::AudioData { data } => {
                assert_eq!(data, vec![0x00, 0x00, 0x00, 0x00]);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_video_packet() {
        let chunk = Chunk::new(0, 0, MessageType::Video, 0, vec![0x00, 0x00, 0x00, 0x00].into());

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::VideoData { data } => {
                assert_eq!(data, vec![0x00, 0x00, 0x00, 0x00]);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_set_chunk_size() {
        let chunk = Chunk::new(0, 0, MessageType::SetChunkSize, 0, vec![0x00, 0xFF, 0xFF, 0xFF].into());

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::SetChunkSize(ProtocolControlMessageSetChunkSize { chunk_size }) => {
                assert_eq!(chunk_size, 0x00FFFFFF);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_window_acknowledgement_size() {
        let chunk = Chunk::new(
            0,
            0,
            MessageType::WindowAcknowledgementSize,
            0,
            vec![0x00, 0xFF, 0xFF, 0xFF].into(),
        );

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::SetAcknowledgementWindowSize(ProtocolControlMessageWindowAcknowledgementSize {
                acknowledgement_window_size,
            }) => {
                assert_eq!(acknowledgement_window_size, 0x00FFFFFF);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_parse_metadata() {
        let mut amf0_writer = Vec::new();

        Amf0Encoder::encode_string(&mut amf0_writer, "onMetaData").unwrap();
        Amf0Encoder::encode_object(&mut amf0_writer, &[("duration".into(), Amf0Value::Number(0.0))]).unwrap();

        let amf_data = Bytes::from(amf0_writer);
        let chunk = Chunk::new(0, 0, MessageType::DataAMF0, 0, amf_data.clone());

        let message = MessageData::read(&chunk).expect("no errors");
        match message {
            MessageData::DataAmf0 { data } => {
                assert_eq!(data, amf_data);
            }
            _ => unreachable!("wrong message type"),
        }
    }

    #[test]
    fn test_unsupported_message_type() {
        let chunk = Chunk::new(0, 0, MessageType(42), 0, vec![0x00, 0x00, 0x00, 0x00].into());

        assert!(matches!(
            MessageData::read(&chunk).expect("no errors"),
            MessageData::Unknown(UnknownMessage {
                msg_type_id: MessageType(42),
                ..
            })
        ));
    }
}
