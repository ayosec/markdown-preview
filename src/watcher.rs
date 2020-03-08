use inotify::{EventMask, Inotify, WatchMask};
use std::path::PathBuf;
use tokio::stream::StreamExt;
use tokio::sync::watch;

pub fn start(opts: &crate::options::Options) -> anyhow::Result<watch::Receiver<()>> {
    let (tx, rx) = watch::channel(());

    let path = PathBuf::from(opts.source.as_str());
    tokio::spawn(async move {
        if let Err(e) = watch_loop(path, tx).await {
            eprintln!("ERROR watch_loop: {:?}", e);
        }
    });

    Ok(rx)
}

async fn watch_loop(path: PathBuf, tx: watch::Sender<()>) -> anyhow::Result<()> {
    let mut inotify = Inotify::init()?;
    let file_name = path.file_name().map(|f| f.to_owned());

    if let Some(parent) = path.parent() {
        inotify.add_watch(parent, WatchMask::CREATE)?;
    }

    inotify.add_watch(&path, WatchMask::MODIFY)?;

    let mut buffer = [0; 32];
    let mut stream = inotify.event_stream(&mut buffer)?;

    while let Some(Ok(event)) = stream.next().await {
        if event.mask.contains(EventMask::MODIFY) {
            tx.broadcast(())?;
        }

        if event.mask.contains(EventMask::CREATE) && event.name == file_name {
            inotify.add_watch(&path, WatchMask::MODIFY)?;
            tx.broadcast(())?;
        }
    }

    anyhow::bail!("Loop stopped")
}
