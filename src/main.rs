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
use iron::prelude::*;
use persistent::State;
use clap::{Arg, App};

mod proto;
mod tracker;
mod util;
mod handlers;

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

    let mut chain = Chain::new(router);
    chain.link(State::<handlers::TorrentList>::both(torrents));

    let listen = format!("{}:{}", config.core.bindaddress, config.core.port);

    Iron::new(chain).http(listen.as_str()).unwrap();
}

