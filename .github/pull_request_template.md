## Summary

<!-- What changed and why? -->

## SDK compatibility

- [ ] This change is backward compatible for exported Rust SDK APIs.
- [ ] This change may be source-incompatible and has maintainer approval.
- [ ] README/examples are updated if public usage changed.
- [ ] Error, auth refresh, timeout, TLS, retry, or stream behavior changes are documented.

## Protobuf/API bindings

- [ ] Source `.proto` changes are not added here; API contract changes belong in `mycel-api`.
- [ ] The matching `mycel-api` branch/tag/commit or submodule update is identified when generated bindings change.
- [ ] `crates/mycel/gen/rust/` was regenerated with `make generate` when API contracts changed.
- [ ] Non-Rust generated bindings were not added.

## Rust checks

- [ ] `cargo fmt --check` passes.
- [ ] `cargo test` passes.
- [ ] `cargo build` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes or is not applicable.

## Repository boundaries

- [ ] No daemon implementation code is added.
- [ ] Product-specific application behavior is not added.
- [ ] Secrets, tokens, passwords, and TLS key material are not logged or exposed.

## Notes

<!-- Migration notes, downstream impact, matching mycel-api version, or follow-up work. -->
