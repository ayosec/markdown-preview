use base64;
use comrak::{markdown_to_html, ComrakOptions};
use kuchiki::traits::*;
use kuchiki;
use mime_guess::guess_mime_type;
use options::Options;
use std::error::Error;
use std::fs::File;
use std::io::Read;

const HEADER: &str = "<DOCTYPE html>\n<meta charset=\"utf-8\">\n";

pub fn render_html(opts: &Options) -> String {
    let source: &str = &opts.source;
    let mut html = match read_source(source) {
        Ok(c) => markdown_to_html(&c, &ComrakOptions::default()),
        Err(e) => format!("Can't read '{}': {:?}\n", source, e),
    };

    html.insert_str(0, HEADER);

    // Inject custom stylesheet, if present.

    if let Some(path) = opts.stylesheet.as_ref().map(|s| s.as_ref()) {
        match read_source(path) {
            Ok(css) => {
                let css = format!("<style>{}</style>", css);
                html.insert_str(HEADER.len(), &css);
            }
            Err(e) => eprintln!("Can't open {}: {}", path, e),
        }
    }

    // Detect images used in the document. If any, replace them with 'data:' URIs

    let document = kuchiki::parse_html().one(html.as_str());

    let mut imgs_found = false;

    for css_match in document.select("img").unwrap() {
        let mut attrs = css_match.attributes.borrow_mut();
        let mut new_src = None;

        if let Some(src) = attrs.get("src") {
            if let Ok(bytes) = read_bytes(src) {
                let mime = guess_mime_type(src);
                new_src = Some(format!("data:{};base64,{}", mime, base64::encode(&bytes)));
                imgs_found = true;
            }
        }

        if let Some(src) = new_src {
            attrs.insert("src", src);
        }
    }

    if imgs_found {
        let mut html = Vec::new();
        document.serialize(&mut html).unwrap();
        String::from_utf8(html).unwrap()
    } else {
        html
    }
}

fn read_source(source: &str) -> Result<String, Box<Error>> {
    Ok(String::from_utf8(read_bytes(source)?)?)
}

fn read_bytes(source: &str) -> Result<Vec<u8>, Box<Error>> {
    let mut file = File::open(source)?;
    let mut content = match file.metadata() {
        Ok(md) => Vec::with_capacity(md.len() as usize),
        Err(_) => Vec::new(),
    };
    file.read_to_end(&mut content)?;
    Ok(content)
}
