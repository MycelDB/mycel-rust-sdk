# Changelog

All notable changes to the Mycel Rust SDK should be documented in this file.

This project follows the spirit of [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and Cargo semantic versioning. Before `1.0.0`, exported helper APIs may still evolve, but source-incompatible changes should be called out clearly.

## [Unreleased]

## [v0.9.0] - 2026-08-31

### Added

- First public-release baseline for the MycelDB Rust SDK.
- Open-source project documentation: contributing guide, security policy, code of conduct, changelog, pull request template, issue templates, and README badges.
- README environment configuration table with variable defaults and descriptions.
- Committed Rust `prost`/`tonic` generated bindings under `crates/mycel/gen/rust/` so normal Cargo builds do not require an external `mycel-api` checkout.

### Changed

- Renamed the low-level generated bindings crate from `mycel-proto` to `mycel`.
- Documented generated-binding policy, SDK compatibility expectations, and Rust validation checks.

## Release notes policy

For each release, add a dated section such as:

```md
## [v0.9.0] - YYYY-MM-DD

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
```

Include compatibility notes for exported Rust SDK helpers, authentication behavior, timeout/retry behavior, TLS behavior, generated API bindings, and any required matching `mycel-api` tag or commit.
