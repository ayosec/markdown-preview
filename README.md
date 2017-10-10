# markdown-preview

A simple tool to see rendered Markdown files in a web browser.

## Usage

Build and install the binary:

    $ cargo build --release
    $ sudo cp ./target/release/markdown-preview  /usr/local/bin/

Then, launch it with any markdown file

    $ markdown-preview foo.md

Finally, open a web browser in `http://localhost:8081`. Address can be modified with `-a` and `-p` arguments.
