# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2022-11-05

- Support for the `PORT` environment variable.
- Human cache time input (e.g. `1h`, `1year 6months`).
- `--anonymize` flag to hide the `Server` and `X-Powered-By` headers.
- Added `zstd` compression support.
- Dynamic cache control (ETag, Last-Modified, Cache-Control).
- Auto-served `index.html` files.
- Use `println` over tracing for trivial logs.

## [0.1.1] - 2022-10-30

> Release Page: <https://github.com/miraclx/zy/releases/tag/v0.1.1>

[unreleased]: https://github.com/miraclx/zy/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/miraclx/zy/releases/tag/v0.1.1
