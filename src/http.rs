use comrak::{markdown_to_html, ComrakOptions};
use std::fs::File;
use std::io::{self, Read};
use tiny_http::Request;
use tiny_http::Response;

pub fn handle_request(request: Request, source: &str) {
    let html = match read_source(source) {
        Ok(c) => markdown_to_html(&c, &ComrakOptions::default()),
        Err(e) => format!("Can't read '{}': {:?}\n", source, e),
    };

    let result = request.respond(Response::from_data(html));

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
