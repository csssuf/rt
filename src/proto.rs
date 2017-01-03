use rustc_serialize::Encodable;
use rustc_serialize::Encoder;
use bencode::encode;

#[derive(RustcEncodable)]
pub struct Peer {
    pub peer_id: String,
    pub ip: String,
    pub port: u16
}

pub struct RTResponse {
    pub interval: u32,
    pub tracker_id: String,
    pub complete: u32,
    pub incomplete: u32,
    pub peers: Vec<Peer>
}

impl Encodable for RTResponse {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("RTResponse", 5, |s| {
            try!(s.emit_struct_field("interval", 0, |s| {
                s.emit_u32(self.interval)
            }));
            try!(s.emit_struct_field("tracker id", 1, |s| {
                s.emit_str(&self.tracker_id)
            }));
            try!(s.emit_struct_field("complete", 2, |s| {
                s.emit_u32(self.complete)
            }));
            try!(s.emit_struct_field("incomplete", 3, |s| {
                s.emit_u32(self.incomplete)
            }));
            try!(s.emit_struct_field("peers", 4, |s| {
                self.peers.encode(s)
            }));
            Ok(())
        })
    }
}

struct RTResponseWarning {
    pub warning_message: String,
    pub interval: u32,
    pub tracker_id: String,
    pub complete: u32,
    pub incomplete: u32,
    pub peers: Vec<Peer>
}

impl Encodable for RTResponseWarning {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("RTResponseWarning", 6, |s| {
            try!(s.emit_struct_field("warning message", 0, |s| {
                s.emit_str(&self.warning_message)
            }));
            try!(s.emit_struct_field("interval", 1, |s| {
                s.emit_u32(self.interval)
            }));
            try!(s.emit_struct_field("tracker id", 2, |s| {
                s.emit_str(&self.tracker_id)
            }));
            try!(s.emit_struct_field("complete", 3, |s| {
                s.emit_u32(self.complete)
            }));
            try!(s.emit_struct_field("incomplete", 4, |s| {
                s.emit_u32(self.incomplete)
            }));
            try!(s.emit_struct_field("peers", 5, |s| {
                self.peers.encode(s)
            }));
            Ok(())
        })
    }
}

struct RTResponseFailure {
    pub failure_reason: String
}

impl Encodable for RTResponseFailure {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("RTResponseFailure", 1, |s| {
            try!(s.emit_struct_field("failure reason", 0, |s| {
                s.emit_str(&self.failure_reason)
            }));
            Ok(())
        })
    }
}

pub fn generate_failure(s: &str) -> Vec<u8> {
    encode(
        RTResponseFailure {
            failure_reason: s.to_string(),
        }
    ).unwrap()
}

pub fn generate_warning(response: RTResponse, message: &str) -> Vec<u8> {
    encode(
        RTResponseWarning {
            warning_message: message.to_string(),
            interval: response.interval,
            tracker_id: response.tracker_id,
            complete: response.complete,
            incomplete: response.incomplete,
            peers: response.peers
        }
    ).unwrap()
}
