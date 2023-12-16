use downcast_rs::{impl_downcast, Downcast};

use super::{p2p::MsgId, proto};

// pub struct MsgCoder<M: prost::Message> {}

pub trait MsgPack: prost::Message + Downcast {
    fn msg_id(&self) -> MsgId;
}

impl_downcast!(MsgPack);

macro_rules! impl_msg_pack {
    ($ty:ty, $msg_id:expr) => {
        impl MsgPack for $ty {
            fn msg_id(&self) -> MsgId {
                $msg_id
            }
        }
    };
}

pub trait RPCReq: MsgPack + Default {
    type Resp: MsgPack + Default;
}

impl_msg_pack!(proto::metric::RscMetric, 0);
impl_msg_pack!(proto::cache::CacheTrans, 1);
impl_msg_pack!(proto::cache::GetCacheRequest, 2);
impl_msg_pack!(proto::cache::GetCacheResponse, 3);
impl_msg_pack!(proto::cache::GetCacheViewRequest, 4);
impl_msg_pack!(proto::cache::GetCacheViewResponse, 5);

impl RPCReq for proto::cache::GetCacheRequest {
    type Resp = proto::cache::GetCacheResponse;
}
impl RPCReq for proto::cache::GetCacheViewRequest {
    type Resp = proto::cache::GetCacheViewResponse;
}

// impl MsgId for raft::prelude::Message {
//     fn msg_id(&self) -> u32 {
//         0
//     }
// }
// impl MsgPack for raft::prelude::Message {
//     fn msg_id() -> u32 {
//         0
//     }
// }
