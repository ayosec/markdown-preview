extern crate base64;
extern crate comrak;
#[macro_use]
extern crate html5ever;
extern crate kuchiki;
extern crate mime_guess;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate tiny_http;

mod http;
mod options;
mod render;

use structopt::StructOpt;

fn main() {
    let opts = options::Options::from_args();

    if opts.render {
        let output = render::render_html(&opts);
        print!("{}", output);
        return;
    }

    let server = tiny_http::Server::http((opts.address.as_str(), opts.port)).unwrap();

    println!("Server ready in {}", server.server_addr());

    loop {
        match server.recv() {
            Ok(req) => http::handle_request(req, &opts),
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        };
    }
}
