use crate::options::Options;
use crate::render::render_html;
use std::net::ToSocketAddrs;
use warp::Filter;

pub async fn start(opts: Options) -> Result<(), anyhow::Error> {
    let address = format!("{}:{}", opts.address, opts.port)
        .to_socket_addrs()?
        .next()
        .unwrap();

    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {} [{:?}ms]",
            info.method(),
            info.path(),
            info.status(),
            info.elapsed().as_millis()
            );
    });
    let opts = warp::any().map(move || opts.clone());

    let routes = warp::path::end().and(opts).map(|opts| render_html(&opts));

    let (addr, fut) = warp::serve(routes.with(log)).try_bind_ephemeral(address)?;
    println!("HTTP server ready at {}", addr);

    Ok(fut.await)
}
