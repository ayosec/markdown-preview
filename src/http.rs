use comrak::{markdown_to_html, ComrakOptions};
use std::fs::File;
use std::io::{self, Read};
use tiny_http::{Request, Response, Header};

const HEADER: &str = "<DOCTYPE html>\n<meta charset=\"utf-8\">\n";

pub fn handle_request(request: Request, source: &str) {
    let mut html = match read_source(source) {
        Ok(c) => markdown_to_html(&c, &ComrakOptions::default()),
        Err(e) => format!("Can't read '{}': {:?}\n", source, e),
    };

    html.insert_str(0, HEADER);

    println!("Sent {} bytes to {}", html.len(), request.remote_addr());

    let response = Response::from_data(html)
                            .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());

    let result = request.respond(response);

    if let Err(err) = result {
        eprintln!("respond failed: {:?}", err);
    }
}

fn read_source(source: &str) -> io::Result<String> {
    let mut file = File::open(source)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
