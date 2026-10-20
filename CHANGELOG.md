# Changelog

All notable changes to the Mycel Rust SDK should be documented in this file.

This project follows the spirit of [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and Cargo semantic versioning. Before `1.0.0`, exported helper APIs may still evolve, but source-incompatible changes should be called out clearly.

## [Unreleased]

## [v0.20.1] - 2026-10-20

### Fixed

- Regenerated committed Rust protobuf/gRPC bindings from `mycel-api v0.20.1` so session, structured-query, and domain-export comments describe read-only operations as repeatable reads pinned to `base_revision`.

### Changed

- Aligned both Cargo package versions and the API submodule with the immutable `mycel-api v0.20.1` correction.
- Removed crates.io/docs.rs badges, marked both Cargo packages `publish = false`, and clarified Git tag/GitHub source-only distribution.

### Compatibility

- Documentation-only generated-binding correction. Rust APIs and protobuf wire behavior are unchanged from v0.20.0; use with Mycel daemon/API v0.20.1 or later in the v0.20 line for the corrected public contract documentation.
- Distribution remains Git tag/GitHub source release only; neither package is published to crates.io.

## [v0.20.0] - 2026-10-19

### Added

- Regenerated Rust bindings from `mycel-api v0.20.0` for durable graph transaction commit idempotency and graph idempotency diagnostics.
- Added explicit idempotent transaction helpers and an opt-in bounded retry policy for ambiguous commit outcomes.

### Changed

- Aligned both Rust package versions and the API submodule with the coordinated MycelDB v0.20.0 release train.
- Renamed the low-level Cargo package to `myceldb-proto` because the `mycel` crates.io name belongs to an unrelated project; its Rust library/import name remains `mycel` for source compatibility.

### Compatibility

- Best used with Mycel daemon/API v0.20.0. Existing non-idempotent transaction methods remain supported, and retries remain opt-in.

## [v0.19.0] - 2026-10-09

### Added

- Regenerated Rust `prost`/`tonic` bindings for `mycel-api` v0.19.0 after the legacy admin space grant RPC was removed.

### Changed

- Aligned Cargo package versions with the coordinated MycelDB v0.19.0 release train.

### Compatibility

- Best used with Mycel daemon/API v0.19.0 for matching identity scoped space access semantics.

## [v0.18.0] - 2026-10-07

### Added

- Regenerated Rust `prost`/`tonic` bindings for `mycel-api` v0.18.0, including asynchronous cluster backup operation contracts, lifecycle states, cancellation flow, retry hints, and readiness blockers.
- Added `AdminClient` helper methods for async cluster backup start, status, cancellation, listing, and backup-set validation.

### Changed

- Aligned Cargo package versions with the coordinated MycelDB v0.18.0 release train.
- Updated backup helper documentation to steer cluster backup callers toward the asynchronous operation API.

### Compatibility

- Best used with Mycel daemon/API v0.18.0 for matching async cluster backup operation semantics.

## [v0.17.0] - 2026-09-30

### Added

- Regenerated Rust `prost`/`tonic` bindings for `mycel-api` v0.17.0, including client space export job APIs, admin Raft snapshot APIs, and graph checkpoint/index status fields.

### Changed

- Aligned Cargo package versions with the coordinated MycelDB v0.17.0 release train.

## [v0.16.0] - 2026-09-24

### Added

- Regenerated Rust `prost`/`tonic` bindings for `mycel-api` v0.16.0, including graph reference replacement messages and results.

### Changed

- Aligned Cargo package versions with the coordinated MycelDB v0.16.0 release train.

## [v0.15.0] - 2026-09-18

### Added

- Regenerated Rust `prost`/`tonic` bindings for `mycel-api` v0.15.0, including dimensioned cluster readiness fields.

### Changed

- Aligned Cargo package versions with the coordinated MycelDB v0.15.0 release train.

## [v0.12.0] - 2026-09-09

### Added

- Regenerated Rust `prost`/`tonic` bindings for hybrid search API additions, including weighted fusion options, metadata filters, and source diagnostics.

## [v0.11.0] - 2026-09-07

### Added

- Regenerated Rust `prost`/`tonic` bindings for the lexical search APIs and exposed thin `Client.search` and `AdminClient.lexical_maintenance` service clients.

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
