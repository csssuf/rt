extern crate iron;
extern crate persistent;
extern crate bencode;

use std::collections::HashMap;
use std::ops::Deref;
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

pub fn root_handler(_r: &mut Request) -> IronResult<Response> {
    let result: Vec<u8> = ::proto::generate_failure("No torrent specified");
    Ok(Response::with((status::Ok, result)))
}

pub fn announce_handler(r: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};

    let result: Vec<u8>;
    
    let remote_addr = r.remote_addr;
    let params = r.get::<Params>().unwrap();
    let mutex = r.get_mut::<State<TorrentList>>().unwrap();
    println!("{:?}", params);

    match params.find(&["info_hash"]) {
        // First check if it's already in the list so we can decide whether or not to create and
        // insert a new torrent.
        Some(&Value::String(ref info_hash)) => {
            let mut found = false;
            {
                let _list_ro = mutex.read().unwrap();
                let list_ro = _list_ro.deref();

                match list_ro.get(info_hash) {
                    Some(torrent) => {
                        println!("Torrent found: {:?}", torrent);
                        found = true;
                    },
                    None => {
                        println!("Torrent not found, adding.");
                    }
                }
            } // list_ro goes out of scope

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

                    if !found {
                        // If this is a new torrent, create the new torrent and initialize its peer
                        // list
                        let mut new_torrent = ::tracker::Torrent::new(info_hash.clone());
                        new_torrent.peers.push(val.clone());
                        list_rw.insert(info_hash.clone(), new_torrent);
                    }

                    // Get Torrent from list
                    match list_rw.get_mut(info_hash) {
                        Some(torrent) => {
                            if found {
                                // If not new torrent, we need to remove old peers with the same
                                // peer_id
                                torrent.peers.retain(|p| p.peer_id != val.peer_id);

                                // And add new peer
                                torrent.peers.push(val.clone());
                            }

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
