use std::net::SocketAddr;

pub fn sock2str(addr: &SocketAddr) -> String {
    match *addr {
        SocketAddr::V4(ref _addr) => {
            format!("{}", _addr.ip())
        }
        SocketAddr::V6(ref _addr) => {
            format!("{}", _addr.ip())
        }
    }
}
