# Mythian

> The bulan.io Backend Server

## Installation

- First, install Rust and Cargo. See <https://rustup.rs/>.
- Next, clone this repository.

  ```console
  git clone https://github.com/bulan-io/bulan-server.git
  ```

## Usage

- Build the server.

  ```console
  cargo build --release
  ```

  This will export the binary to `target/release/mythian`.

  </details>

- Run the server.

  ```console
  ./target/release/mythian
  ```

  The server will be running on <http://localhost:3000> by default, and will serve the current directory.

  <details>
  <summary> You can view the available configuration options by running <code>./target/release/mythian --help</code>. </summary>

  ```console
  $ ./target/release/mythian --help
  Mythian 0.1.0
  The bulan.io Backend Server

  USAGE:
      mythian [OPTIONS] [DIR]

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
