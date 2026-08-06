# Mycel Rust SDK

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
- Admin backup policy/status/list/trigger/delete helpers
- Admin cluster backup trigger/status/list/validate helpers

## Crates

- `mycel-proto`: generated `prost`/`tonic` protobuf and gRPC clients from the language-independent `mycel-api` protobuf contracts. This repo pins `mycel-api` as a submodule at `third_party/mycel-api`; `MYCEL_API_ROOT=/path/to/mycel-api` can override it.
- `mycel-sdk`: ergonomic client wrapper around the generated clients

## Protobuf generation

The Rust SDK does not commit generated protobuf/gRPC bindings. `crates/mycel-proto/build.rs` discovers all `*.proto` files under the `mycel-api` checkout and generates Rust code into Cargo's build output during `cargo build`/`cargo test`.

By default, it reads the first available API checkout in this order:

1. `MYCEL_API_ROOT=/path/to/mycel-api`
2. `third_party/mycel-api` submodule, pinned to the matching `mycel-api` release or branch
3. sibling `../mycel-api` checkout for local workspace development

For a fresh clone, initialize the submodule before building:

```sh
git submodule update --init --recursive
cargo test
```

Set `MYCEL_API_ROOT` to use a different checkout:

```sh
MYCEL_API_ROOT=/path/to/mycel-api cargo test
```

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

`dial` and `dial_admin` store access-token expiry and refresh tokens returned by login. SDK convenience methods refresh near-expiry tokens automatically. If a protected convenience call fails with `Unauthenticated` because the access token is expired, the SDK refreshes once and retries once. You can also call `refresh`, `refresh_operator`, `logout`, or `logout_operator` directly. Raw generated service clients exposed on `client.*` and `admin.*` still receive bearer-token metadata, but callers using them directly should call refresh helpers themselves.

Transaction operation IDs can be generated client-side and passed when beginning a transaction. They are correlation metadata only, not idempotency keys:

```rust
let operation_id = mycel_sdk::new_operation_id();
let tx = client
    .begin_read_write_transaction_with_operation_id(session_id, operation_id)
    .await?;
let commit = client.commit_transaction_result(tx.transaction_id).await?;
let _ = commit.operation_id;
```

Admin backup helpers wrap `mycel.admin.v1.AdminBackupService`:

```rust
let policy = admin.get_backup_policy().await?;
let status = admin.get_backup_status().await?;
let trigger = admin.trigger_backup("before upgrade").await?;
let cluster = admin.trigger_cluster_backup(
    "before upgrade",
    "/mnt/mycel-backups",
    mycel_proto::admin::v1::BackupArchiveFormat::TarZst,
).await?;
let _ = (policy, status, trigger, cluster);
```

## Environment config

`Config::from_env()` reads:

- `MYCELD_GRPC_ADDR`
- `MYCEL_USERNAME`
- `MYCEL_PASSWORD`
- `MYCEL_ACCESS_TOKEN`
- `MYCEL_ACCESS_TOKEN_EXPIRE_TIME` (RFC3339)
- `MYCEL_REFRESH_TOKEN`
- `MYCEL_REFRESH_BEFORE` (`ms`, `s`, `m`, `h` suffixes; default `30s`)
- `MYCEL_CALL_TIMEOUT` (`ms`, `s`, `m`, `h` suffixes)
- `MYCELD_TLS`
- `MYCELD_TLS_CA_FILE`
- `MYCELD_TLS_SERVER_NAME`
- `MYCELD_TLS_INSECURE_SKIP_VERIFY` *(currently rejected by Rust SDK transport)*
- `MYCELD_TLS_CLIENT_CERT_FILE`
- `MYCELD_TLS_CLIENT_KEY_FILE`
- `MYCEL_CLIENT_NAME`
- `MYCEL_CLIENT_VERSION`
- `MYCEL_CLIENT_PLATFORM`
- `MYCEL_CLIENT_DEVICE_LABEL`
