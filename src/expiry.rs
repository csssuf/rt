use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ExpiryMap {
    expiry_map: BTreeMap<Instant, ::tracker::ExpiryPeer>,
    peer_map: HashMap<::tracker::ExpiryPeer, Instant>
}

impl ExpiryMap {
    pub fn new() -> ExpiryMap {
        ExpiryMap {
            expiry_map: BTreeMap::new(),
            peer_map: HashMap::new()
        }
    }

    pub fn upsert_peer(&mut self, peer: ::tracker::ExpiryPeer) {
        let now = Instant::now();

        if let Some(peer_val) = self.peer_map.get(&peer) {
            self.expiry_map.remove(peer_val);
        }

        self.peer_map.insert(peer.clone(), now);
        self.expiry_map.insert(now, peer.clone());
    }

    pub fn get_expired_peers(&mut self, max_age: Duration) -> Vec<::tracker::ExpiryPeer> {
        let expiry_map_clone = self.expiry_map.clone();
        let expiry_iter = expiry_map_clone.into_iter();
        let mut out = Vec::new();

        let now = Instant::now();
        for (expiry, peer) in expiry_iter {
            if now.duration_since(expiry) < max_age {
                break;
            }
            out.push(peer.clone());
            self.peer_map.remove(&peer);
            self.expiry_map.remove(&expiry);
        }
        out
    }
}
