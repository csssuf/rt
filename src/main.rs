extern crate iron;
#[macro_use] extern crate router;
extern crate rustc_serialize;
extern crate bencode;

use iron::prelude::*;
use iron::status;
use router::Router;
use bencode::encode;

mod rtresponse;
mod peer;

fn main() {
    let router = router!{
        get_slash: get "/" => root_handler
    };

    Iron::new(router).http("localhost:3000").unwrap();

    fn root_handler(r: &mut Request) -> IronResult<Response> {
        let s = rtresponse::RTResponse {
                             failure_reason: "".to_string(),
                             warning_message: "".to_string(),
                             interval: 1,
                             min_interval: 0,
                             tracker_id: "t_id".to_string(),
                             complete: 1,
                             incomplete: 2,
                             peers: vec!(peer::Peer { peer_id: "abc".to_string(), ip: "1.2.3.4".to_string(), port: 80 })
                           };
        let result: Vec<u8> = encode(&s).unwrap();
        Ok(Response::with((status::Ok, result)))
    }
}
