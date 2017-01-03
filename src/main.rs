extern crate iron;
#[macro_use] extern crate router;
extern crate rustc_serialize;
extern crate bencode;
extern crate params;

use iron::prelude::*;
use iron::status;
use router::Router;
use bencode::encode;

mod proto;

fn main() {
    let router = router!{
        get_slash: get "/" => root_handler,
        announce: get "/announce" => announce_handler
    };

    Iron::new(router).http("localhost:3000").unwrap();
}

fn root_handler(r: &mut Request) -> IronResult<Response> {
    let result: Vec<u8> = proto::generate_failure("No torrent specified");
    Ok(Response::with((status::Ok, result)))
}

fn announce_handler(r: &mut Request) -> IronResult<Response> {
    use params::{Params, Value};
    let params = r.get_ref::<Params>().unwrap();
    println!("{:?}", params);
    Ok(Response::with((status::Ok, proto::generate_failure("Unimplemented"))))
}
