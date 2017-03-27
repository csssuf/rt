extern crate iron;
extern crate persistent;
extern crate bencode;

use std::collections::HashMap;
use iron::prelude::*;
use iron::typemap::Key;
use iron::status;
use persistent::State;
use bencode::encode;

#[derive(Clone, Copy)]
pub struct TorrentList;

impl Key for TorrentList {
    type Value = HashMap<String, ::tracker::Torrent>;
}

#[derive(Clone, Copy)]
pub struct ExpiryList;

impl Key for ExpiryList {
    type Value = ::expiry::ExpiryMap;
}

pub fn root_handler(_r: &mut Request) -> IronResult<Response> {
    let result: Vec<u8> = ::proto::generate_failure("No torrent specified");
    Ok(Response::with((status::Ok, result)))
}

pub fn announce_handler(r: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};

    let result: Vec<u8>;
    
    let remote_addr = r.remote_addr;
    let params = r.get::<Params>().unwrap();
    let expiry_mutex = r.get::<State<ExpiryList>>().unwrap();
    let mutex = r.get_mut::<State<TorrentList>>().unwrap();
    println!("{:?}", params);

    match params.find(&["info_hash"]) {
        // First check if it's already in the list so we can decide whether or not to create and
        // insert a new torrent.
        Some(&Value::String(ref info_hash)) => {
            // Generate new peer for this call to /announce
            let new_peer = ::tracker::RTPeer::new(
                params.find(&["peer_id"]),
                params.find(&["port"]),
                params.find(&["uploaded"]),
                params.find(&["downloaded"]),
                params.find(&["left"]),
                Some(::util::sock2str(&remote_addr)),
                params.find(&["key"])
            );

            // Make sure RTPeer::new succeeded
            match new_peer {
                Ok(val) => {
                    let mut list_rw = mutex.write().unwrap();

                    if !list_rw.contains_key(info_hash) {
                        // If this is a new torrent, create the new torrent and initialize its peer
                        // list
                        let new_torrent = ::tracker::Torrent::new(info_hash.clone());
                        list_rw.insert(info_hash.clone(), new_torrent);
                    }

                    // Get Torrent from list
                    match list_rw.get_mut(info_hash) {
                        Some(torrent) => {
                            // Remove old peers with the same ID; push new peer
                            torrent.peers.retain(|p| p.peer_id != val.peer_id);
                            torrent.peers.push(val.clone());

                            // Calculate complete and incomplete peers
                            let complete = torrent.peers.to_vec()
                                                        .into_iter()
                                                        .filter(|v| v.left == 0)
                                                        .collect::<Vec<_>>()
                                                        .len();
                            let incomplete = torrent.peers.len() - complete;

                            // Build RTResponse
                            let result_struct = ::proto::RTResponse {
                                interval: 1800,
                                tracker_id: "something".to_string(),
                                complete: complete as u32,
                                incomplete: incomplete as u32,
                                peers: torrent.peers.to_vec()
                                                    .into_iter()
                                                    .map(|v| ::proto::Peer {
                                                        peer_id: v.peer_id,
                                                        ip: v.ip,
                                                        port: v.port
                                                    })
                                                    .collect::<Vec<_>>()
                            };
                            
                            result = encode(result_struct).unwrap();
                        },
                        None => {
                            result = ::proto::generate_failure("Couldn't get torrent info - something went horribly wrong.");
                        }
                    }

                    // Insert ExpiryPeer too
                    if let Ok(new_expiry_peer) = ::tracker::ExpiryPeer::new(params.find(&["peer_id"]),
                                                                            info_hash.clone()) {
                        let mut expiry_map = expiry_mutex.write().unwrap();
                        expiry_map.upsert_peer(new_expiry_peer);
                    }
                },
                _ => { result = ::proto::generate_failure("Error generating peer"); }
            }
        },
        _ => {
            result = ::proto::generate_failure("No info_hash specified");
        }
    }

    Ok(Response::with((status::Ok, result)))
}

pub fn stats_handler(r: &mut Request) -> IronResult<Response> {
    let torrent_list = r.get_mut::<State<TorrentList>>().unwrap().read().unwrap();

    let peer_list_list = torrent_list.values()
                                     .map(|torrent| torrent.peers.clone())
                                     .collect::<Vec<_>>();
    let peerno = peer_list_list.into_iter()
                               .fold(0, |acc, peerlist| acc + peerlist.len());

    let result = format!("{} torrents with {} peers.", torrent_list.len(), peerno);
    Ok(Response::with((status::Ok, result)))
}
