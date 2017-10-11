use options::Options;
use render::render_html;
use tiny_http::{Request, Response, Header};

pub fn handle_request(request: Request, opts: &Options) {

    if request.url() != "/" {
        println!("[{}] rejected", request.url());
        let _ = request.respond(Response::from_string("Not found\n").with_status_code(404));
        return;
    }

    let html = render_html(opts);

    println!("[{}] sent {} bytes to {}", request.url(), html.len(), request.remote_addr());

    let response = Response::from_data(html)
                            .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());

    let result = request.respond(response);

    if let Err(err) = result {
        eprintln!("respond failed: {:?}", err);
    }
}
