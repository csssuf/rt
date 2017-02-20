extern crate iron;
#[macro_use] extern crate router;
extern crate rustc_serialize;
extern crate bencode;
extern crate params;
extern crate persistent;

use std::collections::HashMap;
use std::ops::Deref;
use iron::prelude::*;
use iron::typemap::Key;
use iron::status;
use persistent::State;
use bencode::encode;

mod proto;
mod tracker;
mod util;

#[derive(Clone, Copy)]
pub struct TorrentList;

impl Key for TorrentList {
    type Value = HashMap<String, tracker::Torrent>;
}

fn main() {
    let router = router!{
        get_slash: get "/" => root_handler,
        announce: get "/announce" => announce_handler
    };

    let mut chain = Chain::new(router);
    chain.link(State::<TorrentList>::both(HashMap::new()));

    Iron::new(chain).http("0.0.0.0:3000").unwrap();
}

fn root_handler(r: &mut Request) -> IronResult<Response> {
    let result: Vec<u8> = proto::generate_failure("No torrent specified");
    Ok(Response::with((status::Ok, result)))
}

fn announce_handler(r: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};

    let mut result: Vec<u8> = vec![];
    
    let remote_addr = r.remote_addr;
    let params = r.get::<Params>().unwrap();
    let mutex = r.get_mut::<State<TorrentList>>().unwrap();
    println!("{:?}", params);

    match params.find(&["info_hash"]) {
        Some(&Value::String(ref info_hash)) => {
            let mut done = false;
            {
                let _list_ro = mutex.read().unwrap();
                let list_ro = _list_ro.deref();

                match list_ro.get(info_hash) {
                    Some(torrent) => {
                        println!("Torrent found: {:?}", torrent);
                        // Calculate complete and incomplete
                        let _peer_copy = torrent.peers.to_vec();
                        let complete = _peer_copy.into_iter().filter(|v| v.left == 0).collect::<Vec<_>>().len();
                        let incomplete = torrent.peers.len() - complete;

                        // build result struct
                        let result_struct = proto::RTResponse {
                            interval: 3600,
                            tracker_id: "something".to_string(),
                            complete: complete as u32,
                            incomplete: incomplete as u32,
                            peers: torrent.peers.to_vec().into_iter().map(|v| proto::Peer { peer_id: v.peer_id, ip: v.ip, port: v.port }).collect::<Vec<_>>()
                        };
                        result = encode(result_struct).unwrap();
                        done = true;
                    },
                    None => {
                        println!("Torrent not found, adding.");
                    }
                }
            }
            if !done {
                let mut new_torrent = tracker::Torrent::new(info_hash.clone());
                
                let new_peer = tracker::RTPeer::new(
                    params.find(&["peer_id"]),
                    params.find(&["port"]),
                    params.find(&["uploaded"]),
                    params.find(&["downloaded"]),
                    params.find(&["left"]),
                    Some(util::sock2str(&remote_addr)),
                    params.find(&["key"])
                );

                match new_peer {
                    Ok(val) => {
                        new_torrent.peers.push(val);
                        let mut list_rw = mutex.write().unwrap();
                        list_rw.insert(info_hash.clone(), new_torrent);
                        result = proto::generate_failure("It probably actually worked");
                    },
                    _ => { result = proto::generate_failure("Error generating peer"); }
                }
            }
        },
        _ => {
            result = proto::generate_failure("No info_hash specified");
        }
    }

    Ok(Response::with((status::Ok, result)))
}
