extern crate iron;
#[macro_use] extern crate router;
extern crate rustc_serialize;
extern crate bencode;
extern crate params;
extern crate persistent;
extern crate clap;
#[macro_use] extern crate serde_derive;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::Path;
use iron::prelude::*;
use iron::typemap::Key;
use iron::status;
use persistent::State;
use bencode::encode;
use clap::{Arg, App};

mod proto;
mod tracker;
mod util;

#[derive(Clone, Copy)]
pub struct TorrentList;

impl Key for TorrentList {
    type Value = HashMap<String, tracker::Torrent>;
}

fn main() {
    let opt_matches = App::new("rt")
                            .version("0.1")
                            .author("James Forcier <csssuf@csssuf.net>")
                            .about("bittorrent tracker")
                            .arg(Arg::with_name("config")
                                 .short("c")
                                 .long("config")
                                 .value_name("FILE")
                                 .help("Config file to use")
                                 .takes_value(true))
                            .get_matches();

    let config_path_str = opt_matches.value_of("config").unwrap_or("rt.conf");

    let config_path = Path::new(config_path_str);
    let config_path_disp = config_path.display();
    let mut config_file = match File::open(&config_path) {
        Err(reason) => panic!("couldn't open {}: {}", config_path_disp, reason.description()),
        Ok(file) => file
    };

    let mut config_str = String::new();
    match config_file.read_to_string(&mut config_str) {
        Err(reason) => panic!("couldn't read {}: {}", config_path_disp, reason.description()),
        Ok(_) => println!("Using config file {}", config_path_disp),
    }

    let config = util::parse_config(&config_str);

    let router = router!{
        get_slash: get "/" => root_handler,
        announce: get "/announce" => announce_handler
    };

    let mut chain = Chain::new(router);
    chain.link(State::<TorrentList>::both(HashMap::new()));

    let listen = format!("{}:{}", config.core.bindaddress, config.core.port);

    Iron::new(chain).http(listen.as_str()).unwrap();
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
            let new_peer = tracker::RTPeer::new(
                params.find(&["peer_id"]),
                params.find(&["port"]),
                params.find(&["uploaded"]),
                params.find(&["downloaded"]),
                params.find(&["left"]),
                Some(util::sock2str(&remote_addr)),
                params.find(&["key"])
            );

            // Make sure RTPeer::new succeeded
            match new_peer {
                Ok(val) => {
                    let mut list_rw = mutex.write().unwrap();

                    if !found {
                        // If this is a new torrent, create the new torrent and initialize its peer
                        // list
                        let mut new_torrent = tracker::Torrent::new(info_hash.clone());
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
                            let result_struct = proto::RTResponse {
                                interval: 1800,
                                tracker_id: "something".to_string(),
                                complete: complete as u32,
                                incomplete: incomplete as u32,
                                peers: torrent.peers.to_vec()
                                                    .into_iter()
                                                    .map(|v| proto::Peer {
                                                        peer_id: v.peer_id,
                                                        ip: v.ip,
                                                        port: v.port
                                                    })
                                                    .collect::<Vec<_>>()
                            };
                            
                            result = encode(result_struct).unwrap();
                        },
                        None => {
                            result = proto::generate_failure("Couldn't get torrent info - something went horribly wrong.");
                        }
                    }

                },
                _ => { result = proto::generate_failure("Error generating peer"); }
            }
        },
        _ => {
            result = proto::generate_failure("No info_hash specified");
        }
    }

    Ok(Response::with((status::Ok, result)))
}
