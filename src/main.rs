#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate structopt_derive;

mod http;
mod options;
mod render;

use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = options::Options::from_args();

    if opts.render {
        let output = render::render_html(&opts);
        print!("{}", output);
        return;
    }

    if let Err(e) = http::start(opts).await {
        eprintln!("ERROR: {:?}", e);
    }
}
