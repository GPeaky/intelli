use super::{ChunkPacketHeader, PacketHeader};
use ntex::util::{Bytes, BytesMut};
use prost::Message;

pub struct ToProtoMessageBatched {}

impl ToProtoMessageBatched {
    #[inline(always)]
    pub fn batched_encoded(packets: Vec<PacketHeader>) -> Option<Bytes> {
        let data = ChunkPacketHeader { packets };
        // Todo: Check the data.encoded_len() function
        let mut buf = BytesMut::with_capacity(data.encoded_len());

        data.encode_raw(&mut buf);
        Some(buf.freeze())
    }
}
