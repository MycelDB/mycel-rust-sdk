# Contributing to Mycel Rust SDK

Thank you for contributing to the MycelDB Rust SDK. This repository provides Rust crates and convenience helpers for the MycelDB daemon APIs defined in [`mycel-api`](https://github.com/MycelDB/mycel-api).

## Repository scope

This repo **does** contain:

- `mycel-sdk`: ergonomic Rust client wrappers for connection, authentication, sessions, transactions, graph/query helpers, backup helpers, and watch helpers.
- `mycel`: committed `prost`/`tonic` bindings generated from `mycel-api` protobuf contracts.
- Tests and examples for Rust SDK behavior.
- Build scripts for generating bindings from `mycel-api`.

This repo **does not** contain:

- The MycelDB daemon/server implementation.
- The source `.proto` API contract. API contract changes belong in `mycel-api` first.
- Generated bindings for other languages.
- Product-specific application code.

## Local setup

Normal builds use committed generated bindings under `crates/mycel/gen/rust/` and do not require a `mycel-api` checkout.

Initialize the pinned API submodule only when you need to regenerate bindings from the default checkout:

```sh
git submodule update --init --recursive
```

## Local validation

Run the full CI target before opening a PR:

```sh
make ci
```

This runs:

```sh
cargo fmt --check
cargo test
cargo build
```

For logic changes, also consider running:

```sh
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```

## Generated protobuf bindings

Generated Rust protobuf/gRPC output is committed under `crates/mycel/gen/rust/` so crate releases are self-contained.

When API contracts change:

1. Land or check out the matching `mycel-api` changes.
2. Update `third_party/mycel-api` or set `MYCEL_API_ROOT` to the intended checkout.
3. Run `make generate` to refresh `crates/mycel/gen/rust/`.
4. Run `make ci`.
5. Commit SDK helper/test updates, regenerated files, and the submodule pointer when the pinned API changes.
6. Document compatibility or migration notes when public SDK behavior changes.

Do not hand-edit files under `crates/mycel/gen/rust/`. Regenerate them from `mycel-api` instead.

## Rust SDK compatibility

The generated bindings follow the protobuf contract from `mycel-api`. SDK helper APIs should be stable and idiomatic Rust.

Guidelines:

- Prefer additive helpers over breaking existing callers.
- Keep exported names, function signatures, and public structs/enums stable unless a breaking change is intentional.
- Preserve async cancellation and timeout behavior.
- Return typed or wrapped errors that callers can match or inspect.
- Avoid hiding authentication, authorization, retry, or stream-resume semantics that callers need to understand.
- Keep operation IDs as correlation metadata only; do not treat them as idempotency keys or credentials.
- Keep generated/protobuf-facing code and ergonomic helper code clearly separated.

Before a stable `1.0.0` release, the crates may still evolve, but PRs should still call out source-incompatible changes. After `1.0.0`, follow Cargo semantic versioning for public API changes.

## Rust coding guidelines

- Run `cargo fmt` on Rust files.
- Add or update tests for new helpers, auth behavior, error handling, stream setup, and query builders.
- Prefer `async` APIs that compose naturally with Tokio and `tonic`.
- Do not block the async runtime; use async I/O or `spawn_blocking` for blocking work when needed.
- Do not log credentials, refresh tokens, bearer tokens, TLS key material, or private data.
- Keep default network behavior safe; insecure TLS settings must remain explicit opt-ins.
- Keep feature flags additive and document default features before publishing crates.
- Do not commit `Cargo.lock` for this library workspace unless the project intentionally changes that policy.

## Pull request expectations

A good SDK PR includes:

- A clear summary of the change and why it belongs in the SDK.
- Tests or an explanation for why tests are not applicable.
- `make ci` results.
- The matching `mycel-api` branch/tag/commit when protobuf contracts changed.
- README or example updates when public usage changes.
- Compatibility notes for exported helper/API changes.

## Security issues

Do not report security vulnerabilities in public issues. See [SECURITY.md](SECURITY.md) for private reporting instructions.
