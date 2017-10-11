extern crate base64;
extern crate comrak;
extern crate kuchiki;
extern crate mime_guess;
extern crate structopt;
extern crate tiny_http;
#[macro_use]
extern crate structopt_derive;

mod http;
mod options;
mod render;

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
