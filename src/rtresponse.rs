use rustc_serialize::Encodable;
use rustc_serialize::Encoder;

use peer;

pub struct RTResponse {
    pub failure_reason: String,
    pub warning_message: String,
    pub interval: u32,
    pub min_interval: u32,
    pub tracker_id: String,
    pub complete: u32,
    pub incomplete: u32,
    pub peers: Vec<peer::Peer>
}

impl Encodable for RTResponse {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("RTResponse", 8, |s| {
            try!(s.emit_struct_field("failure reason", 0, |s| {
                s.emit_str(&self.failure_reason)
            }));
            try!(s.emit_struct_field("warning message", 1, |s| {
                s.emit_str(&self.warning_message)
            }));
            try!(s.emit_struct_field("interval", 2, |s| {
                s.emit_u32(self.interval)
            }));
            try!(s.emit_struct_field("min interval", 3, |s| {
                s.emit_u32(self.min_interval)
            }));
            try!(s.emit_struct_field("tracker id", 4, |s| {
                s.emit_str(&self.tracker_id)
            }));
            try!(s.emit_struct_field("complete", 5, |s| {
                s.emit_u32(self.complete)
            }));
            try!(s.emit_struct_field("incomplete", 6, |s| {
                s.emit_u32(self.incomplete)
            }));
            try!(s.emit_struct_field("peers", 7, |s| {
                self.peers.encode(s)
            }));
            Ok(())
        })
    }
}
