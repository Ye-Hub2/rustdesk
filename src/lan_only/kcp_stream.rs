use hbb_common::{
    anyhow,
    bytes::BytesMut,
    tokio::net::UdpSocket,
    ResultType, Stream,
};
use std::sync::Arc;

pub struct KcpStream {
    _private: (),
}

impl KcpStream {
    pub fn peer_silent_for(&self) -> Option<std::time::Duration> {
        None
    }

    pub async fn accept(
        _udp_socket: Arc<UdpSocket>,
        _timeout: std::time::Duration,
        _init_packet: Option<BytesMut>,
    ) -> ResultType<(Self, Stream)> {
        Err(anyhow::anyhow!(
            "UDP hole punching is not supported in lan-only builds"
        ))
    }

    pub async fn connect(
        _udp_socket: Arc<UdpSocket>,
        _timeout: std::time::Duration,
    ) -> ResultType<(Self, Stream)> {
        Err(anyhow::anyhow!(
            "UDP hole punching is not supported in lan-only builds"
        ))
    }
}
