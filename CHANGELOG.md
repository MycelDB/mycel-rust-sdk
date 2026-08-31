# Changelog

All notable changes to the Mycel Rust SDK should be documented in this file.

This project follows the spirit of [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and Cargo semantic versioning. Before `1.0.0`, exported helper APIs may still evolve, but source-incompatible changes should be called out clearly.

## [Unreleased]

### Added

- Open-source project documentation: contributing guide, security policy, code of conduct, pull request template, and issue templates.
- README badges and environment configuration table with variable defaults and descriptions.

## Release notes policy

For each release, add a dated section such as:

```md
## [v0.8.0] - YYYY-MM-DD

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
```

Include compatibility notes for exported Rust SDK helpers, authentication behavior, timeout/retry behavior, TLS behavior, generated API bindings, and any required matching `mycel-api` tag or commit.
