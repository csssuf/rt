extern crate iron;
#[macro_use] extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;

fn main() {
    let router = router!{
        get_slash: get "/" => root_handler
    };

    Iron::new(router).http("localhost:3000").unwrap();

    fn root_handler(r: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello, world")))
    }
}
