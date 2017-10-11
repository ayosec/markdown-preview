use base64;
use comrak::{markdown_to_html, ComrakOptions};
use kuchiki::traits::*;
use kuchiki;
use mime_guess::guess_mime_type;
use std::fs::File;
use std::io::{self, Read};

const HEADER: &str = "<DOCTYPE html>\n<meta charset=\"utf-8\">\n";

pub fn render_html(source: &str, stylesheet: Option<&str>) -> String {
    let mut html = match read_source(source) {
        Ok(c) => markdown_to_html(&c, &ComrakOptions::default()),
        Err(e) => format!("Can't read '{}': {:?}\n", source, e),
    };

    html.insert_str(0, HEADER);

    if let Some(path) = stylesheet {
        match read_source(path) {
            Ok(css) => {
                let css = format!("<style>{}</style>", css);
                html.insert_str(HEADER.len(), &css);
            }
            Err(e) => eprintln!("Can't open {}: {}", path, e),
        }
    }

    // Detect images used in the document. If any, replace them with data: URISs

    let document = kuchiki::parse_html().one(html.as_str());

    let mut imgs_found = 0;

    for css_match in document.select("img").unwrap() {
        let mut attrs = css_match.attributes.borrow_mut();
        let mut new_src = None;

        if let Some(src) = attrs.get("src") {
            imgs_found += 1;

            if let Ok(bytes) = read_bytes(src) {
                let mime = guess_mime_type(src);
                new_src = Some(format!("data:{}/;base64,{}", mime, base64::encode(&bytes)));
            }
        }

        if let Some(src) = new_src {
            attrs.insert("src", src);
        }
    }

    if imgs_found == 0 {
        html
    } else {
        let mut html = Vec::new();
        document.serialize(&mut html).unwrap();
        String::from_utf8(html).unwrap()
    }
}

fn read_source(source: &str) -> io::Result<String> {
    let mut file = File::open(source)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn read_bytes(source: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(source)?;
    let mut content = match file.metadata() {
        Ok(md) => { println!("File {} bytes", md.len()); Vec::with_capacity(md.len() as usize) },
        Err(_) => Vec::new(),
    };
    file.read_to_end(&mut content)?;
    Ok(content)
}
