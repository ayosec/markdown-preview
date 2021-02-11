use crate::options::Options;
use crate::render::render_html;

use std::convert::Infallible;
use std::net::ToSocketAddrs;

use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;
use warp::filters::sse::Event;
use warp::Filter;

pub async fn start(opts: Options, watcher: Option<watch::Receiver<String>>) -> Result<(), anyhow::Error> {
    let address = format!("{}:{}", opts.address, opts.port)
        .to_socket_addrs()?
        .next()
        .unwrap();

    let enable_sse = watcher.is_some();

    let opts = warp::any().map(move || opts.clone());
    let watcher = warp::any().map(move || watcher.clone());

    // GET /
    let root = warp::path::end().and(opts).map(move |opts| {
        let html = render_html(&opts, true, enable_sse);
        warp::http::Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(html)
    });

    // GET /listen
    let listener = warp::path("listen")
        .and(watcher)
        .map(|w| warp::sse::reply(warp::sse::keep_alive().stream(sse_stream(w))));

    let routes = warp::get()
        .and(root.or(listener))
        .with(warp::log::custom(print_request));

    let (addr, fut) = warp::serve(routes).try_bind_ephemeral(address)?;
    println!("HTTP server ready at {}", addr);

    fut.await;
    Ok(())
}

fn print_request(info: warp::log::Info) {
    eprintln!(
        "{} {} {} [{:?}ms]",
        info.method(),
        info.path(),
        info.status(),
        info.elapsed().as_millis()
    );
}

fn sse_stream(watcher: Option<watch::Receiver<String>>) -> impl warp::Stream<Item = Result<Event, Infallible>> {
    WatchStream::new(watcher.unwrap()).map(|html| Ok(Event::default().event("message").data(html)))
}
