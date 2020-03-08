#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate structopt_derive;

mod http;
mod options;
mod render;
mod watcher;

use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = options::Options::from_args();

    if opts.render {
        let output = render::render_html(&opts, true, false);
        print!("{}", output);
        return;
    }

    let watch_stream = watcher::start(&opts).ok();

    if let Err(e) = http::start(opts, watch_stream).await {
        eprintln!("ERROR: {:?}", e);
    }
}
