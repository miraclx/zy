# Zy

> Minimal and blazing-fast file server. For real, this time.

## Features

- [Single Page Application support](https://developer.mozilla.org/en-US/docs/Glossary/SPA)
- Partial responses ([Range](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range) support)
- Cross-Origin Resource Sharing ([CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS))
- [Automatic HTTP compression](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding) (Gzip, Brotli, Deflate)
- [Cache control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) (ETag, Last-Modified, Cache-Control)
- Auto-served `index.html` files
- Sane defaults
  - No access to hidden files
  - No access to content outside the base directory
  - No access to symbolic links outside the base directory

## Installation

To install `zy`, you need Rust `1.59.0` or higher. You can then use `cargo` to build everything:

```console
cargo install zy
```

You can also install the latest version (or a specific commit) of `zy` directly from GitHub.

```console
git clone https://github.com/miraclx/zy.git
cargo install --path zy
```

## Usage

```console
zy
```

_This will start serving your current directory on <http://localhost:3000> by default._

_...you can also specify a different port or base directory:_

```console
zy /path/to/serve
```

_...or perhaps different addresses:_

```console
zy -l 5000 -l 127.0.0.1:8080 -l 192.168.1.25
```

## Configuration

You can run `zy --help` to see all available options.

```console
$ zy --help
Zy 0.1.2
Minimal and blazing-fast file server.

USAGE:
    zy [OPTIONS] [DIR]

ARGS:
    <DIR>    Directory to serve [default: .]

OPTIONS:
    -l, --listen <URI>    Sets the address to listen on (repeatable)
                          Valid: `3000`, `127.0.0.1`, `127.0.0.1:3000` [default: 127.0.0.1:3000]
    -s, --spa             Run as a Single Page Application
    -i, --index <FILE>    Index file to serve from the base directory [default: index.html]
        --404 <FILE>      404 file to serve from the base directory [default: 404.html]
    -c, --cache <SECS>    Cache time (max-age) in seconds [default: 3600]
        --no-cors         Disable Cross-Origin Resource Sharing (CORS)
    -a, --all             Serve hidden files
    -f, --follow-links    Follow symlinks outside of the base directory (unsafe)
    -v, --verbose         Be verbose
    -x, --confirm-exit    Require confirmation before exiting on Ctrl+C
    -h, --help            Print help information
    -V, --version         Print version information
```

## Credits

Zy was originally inspired by [sfz](https://github.com/weihanglo/sfz), [serve](https://github.com/vercel/serve) and [http-server](https://github.com/http-party/http-server). It is written in [Rust](https://rust-lang.org) and uses [actix](https://github.com/actix/actix-web) as the web framework.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
