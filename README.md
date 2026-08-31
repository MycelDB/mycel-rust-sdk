# Mycel Rust SDK

[![CI](https://github.com/MycelDB/mycel-rust-sdk/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/MycelDB/mycel-rust-sdk/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mycel-sdk.svg)](https://crates.io/crates/mycel-sdk)
[![Docs.rs](https://docs.rs/mycel-sdk/badge.svg)](https://docs.rs/mycel-sdk)
[![License](https://img.shields.io/github/license/MycelDB/mycel-rust-sdk)](LICENSE)

Rust connector for MycelDB daemon APIs.

This SDK mirrors the Go SDK shape:

- daemon dial helpers
- plaintext/TLS/mTLS transport config
- user login, refresh, and logout helpers
- operator/admin login, refresh, and logout helpers
- automatic access-token refresh for SDK convenience methods, with one retry on expired-token `Unauthenticated`
- bearer-token metadata injection
- generated Admin and Client service clients from the language-independent `mycel-api` protobuf contracts
- call timeout helpers
- session/transaction helpers
- thin graph/query convenience methods
- structured query builders and explain helpers for common indexed/text/semantic/path/aggregate shapes
- graph-change watch helpers
- Admin backup policy/status/list/trigger/delete helpers
- Admin cluster backup trigger/status/list/validate helpers

## Crates

- `mycel`: committed `prost`/`tonic` protobuf and gRPC clients generated from the language-independent `mycel-api` protobuf contracts.
- `mycel-sdk`: ergonomic client wrapper around the generated clients.

## Protobuf generation

Generated Rust protobuf/gRPC bindings are committed under `crates/mycel/gen/rust/` so tagged crate releases are self-contained. Normal `cargo build` and `cargo test` use the committed generated files and do not require a `mycel-api` checkout.

The current `develop` branch is aligned with `mycel-api` `v0.9.0`.

To regenerate bindings, use a matching `mycel-api` checkout and run:

```sh
MYCEL_API_ROOT=/path/to/mycel-api make generate
```

If `MYCEL_API_ROOT` is not set, generation reads the first available API checkout in this order:

1. `third_party/mycel-api` submodule, pinned to the matching `mycel-api` release or branch
2. sibling `../mycel-api` checkout for local workspace development

Generated files in `crates/mycel/gen/rust/` should be committed when API contracts change.

## Validate

```sh
make ci
```

This runs `cargo fmt --check`, `cargo test`, and `cargo build`.

## Usage

```rust
use mycel_sdk::{dial, Config};

#[tokio::main]
async fn main() -> mycel_sdk::Result<()> {
    let mut client = dial(Config {
        addr: "127.0.0.1:9091".into(),
        username: "alice".into(),
        password: "secret".into(),
        ..Default::default()
    }).await?;

    let me = client.who_am_i().await?;
    println!("{}", me.username);

    Ok(())
}
```

Admin APIs use `dial_admin`:

```rust
let mut admin = mycel_sdk::dial_admin(mycel_sdk::Config {
    addr: "127.0.0.1:9091".into(),
    username: "operator".into(),
    password: "secret".into(),
    ..Default::default()
}).await?;
```

`dial` and `dial_admin` store access-token expiry and refresh tokens returned by login. SDK convenience methods refresh near-expiry tokens automatically. If a protected convenience call fails with `Unauthenticated` because the access token is expired, the SDK refreshes once and retries once. You can also call `refresh`, `logout`, or `get_my_access` directly on client or admin SDK handles. Raw generated service clients exposed on `client.*` and `admin.*` still receive bearer-token metadata, but callers using them directly should call refresh helpers themselves.

Transaction operation IDs can be generated client-side and passed when beginning a transaction. They are correlation metadata only, not idempotency keys:

```rust
let operation_id = mycel_sdk::new_operation_id();
let tx = client
    .begin_read_write_transaction_with_operation_id(session_id, operation_id.clone())
    .await?;
// Perform graph writes against tx.transaction_id.
let commit = client.commit_transaction_result(tx.transaction_id).await?;
let _ = commit.operation_id; // matches operation_id
```

Common structured query shapes can be built without hand-assembling every protobuf field:

```rust
let query = mycel_sdk::query::indexed_node_lookup_query(
    "n",
    "Note",
    "title",
    mycel_sdk::query::string_value("Roadmap"),
    "note",
);
let diagnostics = client.explain_query(tx_id.clone(), query.clone()).await?;
let result = client.execute_query(tx_id, query, 50).await?;
let _ = (diagnostics.plan, result.result);
```

Other builders include `ordered_node_query`, `text_predicate_query`, `semantic_predicate_query`, `path_query`, `aggregate_count_query`, `aggregate_property_query`, and `Client::explain_gql`.

Graph changes can be watched with `GraphChangeService.WatchGraphChanges` through the SDK helper. Persist the last processed `event.revision` and use it as `after_revision` when reconnecting:

```rust
let mut last_revision: i64 = load_checkpoint();
let mut stream = client
    .watch_graph_changes(mycel::client::v1::WatchGraphChangesRequest {
        space_id,
        domain_id,
        after_revision: Some(last_revision),
        include_current: true,
        ..Default::default()
    })
    .await?;
while let Some(msg) = stream.message().await? {
    match msg.message {
        Some(mycel::client::v1::watch_graph_changes_response::Message::Event(event)) => {
            if event.origin.as_ref().map(|origin| origin.operation_id.as_str())
                == Some(operation_id.as_str())
            {
                continue; // optional: ignore a write issued by this workflow
            }
            // Apply event to local cache or derived state.
            last_revision = event.revision;
            save_checkpoint(last_revision);
        }
        Some(mycel::client::v1::watch_graph_changes_response::Message::Gap(_gap)) => {
            // Requested history is unavailable. Rebuild/resync local state,
            // persist a fresh checkpoint, and open a new stream.
            break;
        }
        _ => {}
    }
}
```

Watch helpers refresh/retry only while opening the stream. They do not automatically reconnect or resume if a long-lived stream ends later. Track the last received `event.revision`, reconnect with `after_revision`, and handle `gap` by invalidating or rebuilding local derived state. Dropping the returned stream stops reading; cancel/drop the parent task when stopping early. Global `call_timeout` / `MYCEL_CALL_TIMEOUT` applies to watch streams, so long-lived watchers should usually avoid a short global call timeout.

Admin backup helpers wrap `mycel.admin.v1.AdminBackupService`:

```rust
let policy = admin.get_backup_policy().await?;
let status = admin.get_backup_status().await?;
let trigger = admin.trigger_backup("before upgrade").await?;
let cluster = admin.trigger_cluster_backup(
    "before upgrade",
    "/mnt/mycel-backups",
    mycel::admin::v1::BackupArchiveFormat::TarZst,
).await?;
let _ = (policy, status, trigger, cluster);
```

## Environment config

`Config::from_env()` reads:

| Variable | Default | Description |
| --- | --- | --- |
| `MYCELD_GRPC_ADDR` | `127.0.0.1:9091` | MycelDB daemon gRPC address to dial. |
| `MYCEL_USERNAME` | Empty | Username used for SDK login. |
| `MYCEL_PASSWORD` | Empty | Password used for SDK login. |
| `MYCEL_ACCESS_TOKEN` | Empty | Existing bearer access token to use instead of starting unauthenticated. |
| `MYCEL_ACCESS_TOKEN_EXPIRE_TIME` | `None` | RFC3339 access-token expiry timestamp used to decide when refresh is needed. |
| `MYCEL_REFRESH_TOKEN` | Empty | Refresh token used to renew access tokens. |
| `MYCEL_REFRESH_BEFORE` | `30s` effective default | Duration before access-token expiry when the SDK should proactively refresh; accepts `ms`, `s`, `m`, and `h` suffixes. |
| `MYCEL_CALL_TIMEOUT` | No timeout | Per-RPC timeout when set; accepts `ms`, `s`, `m`, and `h` suffixes. |
| `MYCELD_TLS` | `false` | Enables TLS transport when set to a truthy value. |
| `MYCELD_TLS_CA_FILE` | Empty | Path to a PEM CA bundle used to verify the daemon TLS certificate. |
| `MYCELD_TLS_SERVER_NAME` | Empty | TLS server name override for certificate verification. |
| `MYCELD_TLS_INSECURE_SKIP_VERIFY` | `false` | Parsed from the environment, but currently rejected by Rust SDK transport for safety. |
| `MYCELD_TLS_CLIENT_CERT_FILE` | Empty | Path to the client certificate PEM file for mTLS. |
| `MYCELD_TLS_CLIENT_KEY_FILE` | Empty | Path to the client private key PEM file for mTLS. |
| `MYCEL_CLIENT_NAME` | `mycel-rust-sdk` | Client application name sent in login metadata. |
| `MYCEL_CLIENT_VERSION` | Empty | Client application version sent in login metadata. |
| `MYCEL_CLIENT_PLATFORM` | `rust` | Client platform identifier sent in login metadata. |
| `MYCEL_CLIENT_DEVICE_LABEL` | Empty | Optional device label sent in login metadata. |

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for SDK contribution guidelines, generated binding policy, Rust validation checks, and compatibility expectations. See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) for community standards and [`CHANGELOG.md`](CHANGELOG.md) for release notes.

## Security

Please report suspected vulnerabilities privately through GitHub Security Advisories / private vulnerability reporting. See [`SECURITY.md`](SECURITY.md).

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
