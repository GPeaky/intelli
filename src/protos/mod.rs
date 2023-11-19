include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

use crate::protos::packet_header::PacketType;
use prost::Message;

pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

pub trait ToProtoMessage {
    type ProtoType: Message;
    fn to_proto(&self) -> Option<Self::ProtoType>;

    // TODO: Try to remove packet_type from here
    fn convert_and_encode(&self, packet_type: PacketType) -> Option<Vec<u8>>
    where
        Self: Sized,
    {
        let Some(proto_data) = self.to_proto() else {
            return None;
        };

        let proto_data: Vec<u8> = proto_data.encode_to_vec();

        Some(
            PacketHeader {
                r#type: packet_type.into(),
                payload: proto_data,
            }
            .encode_to_vec(),
        )
    }
}

// TODO: Avoid Cloning & Implementing ToProtoMessage for Vec<Vec<u8>>
impl ToProtoMessage for Vec<Vec<u8>> {
    type ProtoType = ChunkPacketHeader;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        Some(ChunkPacketHeader {
            packets: self.clone(),
        })
    }

    // TODO: Try to remove packet_type from here
    fn convert_and_encode(&self, _packet_type: PacketType) -> Option<Vec<u8>>
    where
        Self: Sized,
    {
        let Some(data) = self.to_proto() else {
            return None;
        };

        Some(data.encode_to_vec())
    }
}
