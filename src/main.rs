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
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use iron::prelude::*;
use persistent::State;
use clap::{Arg, App};

mod proto;
mod tracker;
mod util;
mod handlers;
mod expiry;

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
        get_slash: get "/" => handlers::root_handler,
        announce: get "/announce" => handlers::announce_handler,
        stats: get "/stats" => handlers::stats_handler
    };

    let torrents: Arc<RwLock<HashMap<std::string::String, tracker::Torrent>>>
        = Arc::new(RwLock::new(HashMap::new()));
    let _torrents = torrents.clone();

    let expiry_list: Arc<RwLock<expiry::ExpiryMap>>
        = Arc::new(RwLock::new(expiry::ExpiryMap::new()));
    let _expiry_list = expiry_list.clone();

    let mut chain = Chain::new(router);
    chain.link(State::<handlers::TorrentList>::both(_torrents));
    chain.link(State::<handlers::ExpiryList>::both(_expiry_list));

    let torrents = torrents.clone();
    let announce_time = config.torrents.announcetime;

    // Build a thread that periodically checks for expired peers
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(5000));

            let mut expiry_map = expiry_list.write().unwrap();
            //println!("{:?}", expiry_map.deref());

            let mut torrent_list = torrents.write().unwrap();
            //println!("{:?}", torrent_list.deref());

            for ex_peer in expiry_map.get_expired_peers(Duration::new(announce_time as u64, 0)) {
                torrent_list.remove(&ex_peer.torrent_info_hash);
            }
        }
    });

    let listen = format!("{}:{}", config.core.bindaddress, config.core.port);

    Iron::new(chain).http(listen.as_str()).unwrap();
}

