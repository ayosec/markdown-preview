use base64;
use comrak::{markdown_to_html, ComrakOptions};
use html5ever::{QualName, ns};
use kuchiki::traits::*;
use kuchiki::{self, ExpandedName, NodeRef};
use crate::options::Options;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::iter::Peekable;
use std::process::{Command, Stdio};
use std::str::{self,FromStr};

const HEADER: &str = "<!DOCTYPE html>\n<meta charset=\"utf-8\">\n";

pub fn render_html(opts: &Options) -> String {
    let source: &str = &opts.source;
    let mut html = match read_source(source) {
        Ok(c) => markdown_to_html(&c, &comrak_options()),
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

    let document = kuchiki::parse_html().one(html.as_str());

    process_images(&document);
    process_code_snippets(&document);

    if opts.toc {
        process_toc(&document);
    }

    // Generate final HTML

    let mut html = Vec::new();
    document.serialize(&mut html).unwrap();
    String::from_utf8(html).unwrap()
}

fn comrak_options() -> ComrakOptions {
    let mut cm_opts = ComrakOptions::default();
    cm_opts.ext_strikethrough = true;
    cm_opts.ext_table = true;
    cm_opts.ext_autolink = true;
    cm_opts.ext_tasklist = true;
    cm_opts.ext_superscript = true;
    cm_opts
}

fn read_source(source: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(read_bytes(source)?)?)
}

fn read_bytes(source: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(source)?;
    let mut content = match file.metadata() {
        Ok(md) => Vec::with_capacity(md.len() as usize),
        Err(_) => Vec::new(),
    };
    file.read_to_end(&mut content)?;
    Ok(content)
}

// Detect images used in the document.
// If any, replace them with 'data:' URIs
fn process_images(document: &NodeRef) {
    for css_match in document.select("img").unwrap() {
        let mut attrs = css_match.attributes.borrow_mut();
        let mut new_src = None;

        if let Some(src) = attrs.get("src") {
            if let Ok(bytes) = read_bytes(src) {
                let mime = mime_guess::from_path(src).first_or_octet_stream();
                new_src = Some(format!("data:{};base64,{}", mime, base64::encode(&bytes)));
            }
        }

        if let Some(src) = new_src {
            attrs.insert("src", src);
        }
    }
}

// Detect <pre><code> blocks, and parse them via Pygments
fn process_code_snippets(document: &NodeRef) {
    let mut to_detach = Vec::new();
    for css_match in document.select("pre code").unwrap() {
        if let Some(code_class) = css_match.attributes.borrow().get("class") {
            eprintln!("{:?}", code_class);
            let spawn = Command::new("pygmentize")
                .args(&["-f", "html", "-O", "noclasses", "-l"])
                .arg(code_class.replace("language-", ""))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn();

            let mut process = match spawn {
                Ok(process) => process,
                Err(e) => {
                    eprintln!("Can't launch pygmentize: {:?}", e);
                    continue;
                }
            };

            let code_text = css_match.text_contents();
            let html_code = match process.stdin.as_mut().map(|s| s.write_all(code_text.as_bytes())) {
                Some(Ok(_)) => {
                    let output = process.wait_with_output().map(|o| o.stdout);
                    match output.as_ref().map(|o| str::from_utf8(o.as_ref())) {
                        Ok(Ok(o)) => kuchiki::parse_html().one(o),
                        e => {
                            eprintln!("Can't read HTML from pygmentize: {:?}", e);
                            continue
                        }
                    }
                }
                e => {
                    eprintln!("Can't write to pygmentize: {:?}", e);
                    let _ = process.kill();
                    continue;
                }
            };

            let pre_elem = css_match.as_node().parent().unwrap();
            pre_elem.insert_after(html_code.select_first("div").unwrap().as_node().clone());
            to_detach.push(pre_elem);
        }
    }

    for elem in to_detach {
        elem.detach();
    }
}

macro_rules! qname {
    ($local_name:expr) => {
        QualName::new(None, ns!(), $local_name.into())
    };
}

macro_rules! attribute {
    ($value:expr) => {
        kuchiki::Attribute {
            prefix: None,
            value: $value.into()
        }
    }
}

macro_rules! element {
    ($tag:expr) => {
        NodeRef::new_element(qname!($tag), vec![])
    };

    ($tag:expr, $($key:expr => $value:expr),*) => {
        NodeRef::new_element(qname!($tag), vec![$((ExpandedName::new(ns!(), $key), attribute!($value))),*])
    };
}

struct TocItem {
    level: u8,
    ref_name: String,
    content: Vec<NodeRef>,
}

// Generate a table of contents based on the info in the h# tags
fn process_toc(document: &NodeRef) {
    let mut toc_links = vec![];
    let mut refs_count = 0;

    // Collect all H# tags, and inject a <a name="...">
    for node in document.traverse() {
        if let kuchiki::iter::NodeEdge::Start(ref start) = node {
            if let Some(data) = start.as_element() {
                let tag_name = data.name.local.to_lowercase();
                if tag_name.starts_with('h') {
                    if let Ok(header_level) = u8::from_str(&tag_name[1..]) {
                        // Skip top level, used for titles
                        if header_level > 1 {
                            refs_count += 1;
                            let ref_name = format!("ref-{}", refs_count);

                            toc_links.push(TocItem {
                                level: header_level,
                                ref_name: ref_name.clone(),
                                content: clone_tree(start.children()),
                            });

                            let link = element!("a", "name" => ref_name);
                            start.prepend(link);
                        }
                    }
                }
            }
        }
    }

    // Generate nested lists with the collected headers

    let list = make_toc_lists(&mut toc_links.iter().peekable());
    toc_location(document).prepend(list);
}

fn make_toc_lists<'a, I>(toc_links: &mut Peekable<I>) -> NodeRef
where
    I: Iterator<Item = &'a TocItem>,
{
    let current_list = element!("ol");

    'top: loop {
        let li = element!("li");
        let current_level;

        {
            let toc_link = match toc_links.next() {
                None => break,
                Some(ti) => ti,
            };

            let anchor = element!("a", "href" => format!("#{}", toc_link.ref_name));
            for c in toc_link.content.clone() {
                anchor.append(c);
            }

            li.append(anchor);
            current_list.append(li.clone());

            current_level = toc_link.level;
        }

        'inner: loop {
            // Check if the next item is at a different level
            let next_level = match toc_links.peek() {
                None => break 'top,
                Some(p) => p.level,
            };

            if current_level < next_level {
                let sublist = make_toc_lists(toc_links);
                li.append(sublist);
            } else if current_level > next_level {
                break 'top;
            } else {
                break 'inner;
            }
        }
    }

    current_list
}

fn toc_location(document: &NodeRef) -> NodeRef {
    if let Ok(r) = document.select_first(".toc") {
        return r.as_node().clone();
    }

    if let Ok(r) = document.select_first("body") {
        return r.as_node().clone();
    }

    document.clone()
}

fn clone_tree<I>(nodes: I) -> Vec<NodeRef>
where
    I: Iterator<Item = NodeRef>,
{
    nodes
        .map(|node| {
            let n = NodeRef::new(node.data().clone());
            for child in clone_tree(node.children()) {
                n.append(child);
            }
            n
        })
        .collect()
}
