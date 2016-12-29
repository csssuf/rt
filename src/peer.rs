#[derive(RustcEncodable)]
pub struct Peer {
    pub peer_id: String,
    pub ip: String,
    pub port: u16
}
