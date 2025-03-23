use bytes::Bytes;
use nutype_enum::nutype_enum;

nutype_enum! {
    /// FLV `AACPacketType`
    ///
    /// Defined in the FLV specification. Chapter 1 - AACAUDIODATA
    ///
    /// The AACPacketType indicates the type of data in the AACAUDIODATA.
    pub enum AacPacketType(u8) {
        /// Sequence Header
        SequenceHeader = 0,
        /// Raw
        Raw = 1,
    }
}

/// FLV `AACAUDIODATA` tag
///
/// This is a container for aac data.
/// This enum contains the data for the different types of aac packets.
/// Defined in the FLV specification. Chapter 1 - AACAUDIODATA
#[derive(Debug, Clone, PartialEq)]
pub enum AacAudioData {
    /// AAC Sequence Header
    SequenceHeader(Bytes),
    /// AAC Raw
    Raw(Bytes),
    /// Data we don't know how to parse
    Unknown { aac_packet_type: AacPacketType, data: Bytes },
}

impl AacAudioData {
    /// Create a new AAC packet from the given data and packet type
    pub fn new(aac_packet_type: AacPacketType, data: Bytes) -> Self {
        match aac_packet_type {
            AacPacketType::Raw => AacAudioData::Raw(data),
            AacPacketType::SequenceHeader => AacAudioData::SequenceHeader(data),
            _ => AacAudioData::Unknown { aac_packet_type, data },
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cases = [
            (
                AacPacketType::Raw,
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::Raw(Bytes::from(vec![0, 1, 2, 3])),
            ),
            (
                AacPacketType::SequenceHeader,
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::SequenceHeader(Bytes::from(vec![0, 1, 2, 3])),
            ),
            (
                AacPacketType(0x0),
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::SequenceHeader(Bytes::from(vec![0, 1, 2, 3])),
            ),
            (
                AacPacketType(0x1),
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::Raw(Bytes::from(vec![0, 1, 2, 3])),
            ),
            (
                AacPacketType(0x2),
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::Unknown {
                    aac_packet_type: AacPacketType(0x2),
                    data: Bytes::from(vec![0, 1, 2, 3]),
                },
            ),
            (
                AacPacketType(0x3),
                Bytes::from(vec![0, 1, 2, 3]),
                AacAudioData::Unknown {
                    aac_packet_type: AacPacketType(0x3),
                    data: Bytes::from(vec![0, 1, 2, 3]),
                },
            ),
        ];

        for (packet_type, data, expected) in cases {
            let packet = AacAudioData::new(packet_type, data.clone());
            assert_eq!(packet, expected);
        }
    }

    #[test]
    fn test_aac_packet_type() {
        assert_eq!(
            format!("{:?}", AacPacketType::SequenceHeader),
            "AacPacketType::SequenceHeader"
        );
        assert_eq!(format!("{:?}", AacPacketType::Raw), "AacPacketType::Raw");
        assert_eq!(format!("{:?}", AacPacketType(0x2)), "AacPacketType(2)");
        assert_eq!(format!("{:?}", AacPacketType(0x3)), "AacPacketType(3)");

        assert_eq!(AacPacketType(0x01), AacPacketType::Raw);
        assert_eq!(AacPacketType(0x00), AacPacketType::SequenceHeader);
    }
}
