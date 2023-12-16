pub mod msg_pack;
pub mod p2p;
pub mod p2p_quic;

pub mod proto {
    pub mod metric {
        include!(concat!(env!("OUT_DIR"), "/metric.rs"));
    }
    pub mod cache {
        include!(concat!(env!("OUT_DIR"), "/cache.rs"));
    }
}
