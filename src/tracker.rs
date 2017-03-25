use std::cmp::Ordering;
use std::time::Instant;
use params;
use params::{Value, FromValue};

#[derive(Debug)]
pub struct Torrent {
    pub info_hash: String,
    pub peers: Vec<RTPeer>
}

impl Torrent {
    pub fn new(info_hash: String) -> Torrent {
        Torrent {
            info_hash: info_hash,
            peers: Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct RTPeer {
    pub peer_id: String,
    pub port: u16,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub ip: String,
    key: String
}

impl RTPeer {
    pub fn new(peer_id: Option<&params::Value>,
           port: Option<&params::Value>,
           uploaded: Option<&params::Value>,
           downloaded: Option<&params::Value>,
           left: Option<&params::Value>,
           ip: Option<String>,
           key: Option<&params::Value>) -> Result<RTPeer, &'static str> {
        let _peer_id: String;
        let _port: u16;
        let _uploaded: u32;
        let _downloaded: u32;
        let _left: u32;
        let _ip: String;
        let _key: String;

        match peer_id {
            Some(&Value::String(ref found)) => { _peer_id = found.clone(); },
            _ => { return Err("No peer_id provided"); }
        }
        match port {
            Some(found) => { 
                match u16::from_value(found) {
                    Some(converted) => { _port = converted; },
                    _ => { return Err("No port provided"); }
                }
            },
            _ => { return Err("No port provided"); }
        }
        match uploaded {
            Some(found) => { 
                match u32::from_value(found) {
                    Some(converted) => { _uploaded = converted; },
                    _ => { _uploaded = 0; }
                }
            },
            _ => { _uploaded = 0; }
        }
        match downloaded {
            Some(found) => {
                match u32::from_value(found) {
                    Some(converted) => { _downloaded = converted; },
                    _ => { _downloaded = 0; }
                }
            },
            _ => { _downloaded = 0; }
        }
        match left {
            Some(found) => {
                match u32::from_value(found) {
                    Some(converted) => { _left = converted; },
                    _ => { return Err("Invalid value for 'left'"); }
                }
            },
            _ => { return Err("No left provided"); }
        }
        match ip {
            Some(found) => { _ip = found; },
            _ => { return Err("No ip provided"); }
        }
        match key {
            Some(&Value::String(ref found)) => { _key = found.clone(); }
            _ => { _key = "".to_string(); }
        }
        Ok(RTPeer {
            peer_id: _peer_id,
            port: _port,
            uploaded: _uploaded,
            downloaded: _downloaded,
            left: _left,
            ip: _ip,
            key: _key
        })
    }
}

#[derive(Clone, Debug)]
pub struct ExpiryPeer {
    pub peer_id: String,
    pub torrent_info_hash: String,
    pub expiry: Instant
}

impl ExpiryPeer {
    pub fn new(peer_id: Option<&params::Value>,
            torrent_info_hash: Option<&params::Value>) -> Result<ExpiryPeer, &'static str>{
        let _peer_id: String;
        let _torrent_info_hash: String;

        match peer_id {
            Some(&Value::String(ref found)) => { _peer_id = found.clone(); },
            _ => { return Err("No peer_id provided"); }
        }
        match torrent_info_hash {
            Some(&Value::String(ref found)) => { _torrent_info_hash = found.clone(); },
            _ => { return Err("No torrent_info_hash provided"); }
        }
        Ok(ExpiryPeer {
            peer_id: _peer_id,
            torrent_info_hash: _torrent_info_hash,
            expiry: Instant::now()
        })
    }
}

impl Ord for ExpiryPeer {
    fn cmp(&self, other: &ExpiryPeer) -> Ordering {
        self.expiry.cmp(&other.expiry)
    }
}

impl PartialOrd for ExpiryPeer {
    fn partial_cmp(&self, other: &ExpiryPeer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExpiryPeer {
    fn eq(&self, other: &ExpiryPeer) -> bool {
        self.expiry == other.expiry
    }
}

impl Eq for ExpiryPeer {}
