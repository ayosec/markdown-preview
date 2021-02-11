#[derive(StructOpt, Debug, Clone)]
#[rustfmt::skip]
#[structopt(name = "markdown-preview", about = "Preview for markdown files")]
pub struct Options {
    #[structopt(short = "p", long = "port", help = "Bind port", default_value = "8081")]
    pub port: u16,

    #[structopt(short = "a", long = "address", help = "Bind address", default_value = "127.0.0.1")]
    pub address: String,

    #[structopt(short = "t", long = "toc", help = "Generate a Table of Contents")]
    pub toc: bool,

    #[structopt(short = "r", long = "render", help = "Render HTML and exist")]
    pub render: bool,

    #[structopt(short = "s", long = "stylesheet", help = "Stylesheet to include in the HTML")]
    pub stylesheet: Option<String>,

    #[structopt(help = "Source file")]
    pub source: String,
}
