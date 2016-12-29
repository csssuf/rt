use peer;

#[derive(RustcEncodable)]
pub struct RTResponse {
    pub failure_reason: String,
    pub warning_message: String,
    pub interval: u32,
    pub min_interval: u32,
    pub complete: u32,
    pub incomplete: u32,
    pub peer: Vec<peer::Peer>
}
