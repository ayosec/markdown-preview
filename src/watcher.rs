use std::path::PathBuf;

use inotify::{Inotify, WatchMask};
use tokio::sync::watch;

use crate::options::Options;
use crate::render::render_html;

pub fn start(opts: &Options) -> anyhow::Result<watch::Receiver<String>> {
    let (tx, rx) = watch::channel(String::new());
    let opts = opts.clone();

    tokio::task::spawn_blocking(move || {
        if let Err(e) = watch_loop(opts, tx) {
            eprintln!("ERROR watch_loop: {:?}", e);
        }
    });

    Ok(rx)
}

fn watch_loop(opts: Options, notify: watch::Sender<String>) -> anyhow::Result<()> {
    let source = PathBuf::from(opts.source.as_str()).canonicalize()?;

    let mut inotify = Inotify::init()?;

    inotify.add_watch(source.parent().unwrap(), WatchMask::CREATE)?;
    inotify.add_watch(&source, WatchMask::MODIFY | WatchMask::CREATE)?;

    loop {
        let mut buffer = [0u8; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;

        let mut changed = false;
        for event in events {
            if event.name == source.file_name() {
                changed = true;
            }
        }

        if changed {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let html = render_html(&opts, false, false);
            notify.send(html)?;
        }
    }
}
