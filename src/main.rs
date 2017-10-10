extern crate comrak;
extern crate tiny_http;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

mod http;
mod options;

use structopt::StructOpt;

fn main() {

    let opts = options::Options::from_args();

    let server = tiny_http::Server::http((opts.address.as_str(), opts.port)).unwrap();

    println!("Server ready in {}", server.server_addr());

    loop {
        match server.recv() {
            Ok(req) => http::handle_request(req, &opts.source, opts.stylesheet.as_ref().map(|s| s.as_ref())),
            Err(e) => { println!("error: {}", e); break }
        };
    }

}
