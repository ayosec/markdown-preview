use options::Options;
use render::render_html;
use std::time::Instant;
use tiny_http::{Header, Request, Response};

pub fn handle_request(request: Request, opts: &Options) {
    if request.url() != "/" {
        println!("[{}] rejected", request.url());
        let _ = request.respond(Response::from_string("Not found\n").with_status_code(404));
        return;
    }

    let render_start = Instant::now();

    let html = render_html(opts);

    let duration = {
        let elapsed = render_start.elapsed();
        (elapsed.as_secs() * 1000) as f64 + elapsed.subsec_nanos() as f64 * 1e-6
    };

    println!(
        "[{}] sent {} bytes to {} [{:.4}ms]",
        request.url(),
        html.len(),
        request.remote_addr(),
        duration
    );

    let response = Response::from_data(html)
        .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());

    let result = request.respond(response);

    if let Err(err) = result {
        eprintln!("respond failed: {:?}", err);
    }
}
