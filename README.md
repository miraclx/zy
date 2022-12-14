# Zy

> Minimal and blazing-fast file server. For real, this time.

[![Crates.io](https://img.shields.io/crates/v/zy?label=latest)](https://crates.io/crates/zy)
[![CI](https://github.com/miraclx/zy/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/miraclx/zy/actions/workflows/ci.yml)
[![MIT or Apache 2.0 Licensed](https://img.shields.io/crates/l/zy.svg)](#license)

## Features

- [Single Page Application support](https://developer.mozilla.org/en-US/docs/Glossary/SPA)
- Partial responses ([Range](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range) support)
- Cross-Origin Resource Sharing ([CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS))
- [Automatic HTTP compression](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding) (Zstd, Gzip, Brotli, Deflate)
- Dynamic [cache control](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control) (ETag, Last-Modified, Cache-Control)
- Auto-served `index.html` files
- Sane defaults
  - No access to hidden files
  - No access to content outside the base directory
  - No access to symbolic links outside the base directory

## Installation

You can download any of the pre-compiled binaries from the [releases page](https://github.com/miraclx/zy/releases).

Or if you already have Rust installed, you can install it with `cargo`:

> - Please, note that the minimum supported version of Rust for `zy` is `1.59.0`.
> - Also, that the binary may be bigger than expected because it contains debug symbols. This is intentional. To remove debug symbols and therefore reduce the file size, you can instead run it with the `--profile slim` or simply just run `strip` on it.

```console
cargo install zy
```

Alternatively, you can also build the latest version of `zy` directly from GitHub.

```console
cargo install --git https://github.com/miraclx/zy.git
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
Zy 0.2.0
Minimal and blazing-fast file server.

USAGE:
    zy [OPTIONS] [DIR]

ARGS:
    <DIR>    Directory to serve [default: .]

OPTIONS:
    -l, --listen <URI>    Sets the address to listen on (repeatable) [default: 127.0.0.1:3000]
                          Valid: `3000`, `127.0.0.1`, `127.0.0.1:3000` [env: PORT]
    -s, --spa             Run as a Single Page Application
    -i, --index <FILE>    Index file to serve from the base directory [default: index.html]
        --404 <FILE>      404 file to serve from the base directory [default: 404.html]
    -c, --cache <TIME>    Cache time (max-age) [default: 1h]
                          Valid: `10` for 10 seconds, `1h`, `1year 6months`
        --no-cors         Disable Cross-Origin Resource Sharing (CORS)
    -a, --all             Serve hidden files
    -f, --follow-links    Follow symlinks outside of the base directory (unsafe)
    -v, --verbose         Be verbose
    -x, --confirm-exit    Require confirmation before exiting on Ctrl+C
    -Z, --anonymize       Hide the `Server` and `X-Powered-By` headers [alias: `--anon`]
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
