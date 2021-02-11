use crate::options::Options;
use crate::render::render_html;
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use tokio::stream::StreamExt;
use tokio::sync::{mpsc, watch};
use warp::filters::sse::ServerSentEvent;
use warp::Filter;

type Connections = Arc<Mutex<Vec<mpsc::UnboundedSender<String>>>>;

pub async fn start(opts: Options, watcher: Option<watch::Receiver<()>>) -> Result<(), anyhow::Error> {
    let address = format!("{}:{}", opts.address, opts.port)
        .to_socket_addrs()?
        .next()
        .unwrap();

    let connections = Connections::default();
    let enable_sse = watcher.is_some();

    if let Some(mut watcher) = watcher {
        let connections = connections.clone();
        let opts = opts.clone();

        tokio::spawn(async move {
            while watcher.recv().await.is_some() {
                let html = render_html(&opts, false, false);
                println!("HTML updated ({} bytes)", html.len());
                connections.lock().unwrap().retain(|tx| {
                    // Remove SSE stream if it is gone
                    tx.send(html.clone()).is_ok()
                });
            }
        });
    }

    let opts = warp::any().map(move || opts.clone());
    let connections = warp::any().map(move || connections.clone());

    // GET /
    let root = warp::path::end().and(opts).map(move |opts| {
        let html = render_html(&opts, true, enable_sse);
        warp::http::Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(html)
    });

    // GET /listen
    let listener = warp::path("listen")
        .and(connections)
        .map(|c| warp::sse::reply(warp::sse::keep_alive().stream(sse_stream(c))));

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

fn sse_stream(c: Connections) -> impl warp::Stream<Item = Result<impl ServerSentEvent, Infallible>> {
    let (tx, rx) = mpsc::unbounded_channel();
    c.lock().unwrap().push(tx);
    rx.map(|html| Ok(warp::sse::data(html)))
}
