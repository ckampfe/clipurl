# clipurl

[![Rust](https://github.com/ckampfe/clipurl/actions/workflows/rust.yml/badge.svg)](https://github.com/ckampfe/clipurl/actions/workflows/rust.yml)

Clipurl polls your system clipboard/pasteboard,
checks if its contents parse as a URL (according to [this library](https://crates.io/crates/url)), and if so, adds that URL Sqlite database of your choice.

The intent being to give you a history of the URLs you share.

Inspired by [this](https://lobste.rs/s/b6oms9/this_project_will_only_take_2_hours).

## Install

```
$ cargo install --git https://github.com/ckampfe/clipurl
```

## Typical use

It probably makes the most sense to add it to your init (systemd, launchd, or whatever else), but you can run `clipurl` in a terminal, as a background job, a daemon, or whatever else like so:

```
$ clipurl --links-db-file my_links.db --poll-interval-milliseconds 5000
```

`clipurl` will create the `links-db-file` if it does not exist.

## CLI

```
$ clipurl -h
clipurl 0.1.0

USAGE:
    clipurl [OPTIONS] --links-db-file <links-db-file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --links-db-file <links-db-file>
    -p, --poll-interval-milliseconds <poll-interval-milliseconds>     [default: 5000]
```

## Sqlite

`clipurl` bundles its own copy of Sqlite. This might change in the future.
