# Mycel Rust SDK

Rust connector for MycelDB daemon APIs.

This SDK mirrors the Go SDK shape:

- daemon dial helpers
- plaintext/TLS/mTLS transport config
- user login and refresh helpers
- operator/admin login helper
- bearer-token metadata injection
- generated Admin and Client service clients from the language-independent `mycel-api` protobuf contracts
- call timeout helpers
- session/transaction helpers
- thin graph/query convenience methods
- Admin backup policy/status/list/trigger/delete helpers

## Crates

- `mycel-proto`: generated `prost`/`tonic` protobuf and gRPC clients from the sibling `../mycel-api/api/proto` checkout, or from `MYCEL_API_ROOT=/path/to/mycel-api`
- `mycel-sdk`: ergonomic client wrapper around the generated clients

## Protobuf generation

The Rust SDK does not commit generated protobuf/gRPC bindings. `crates/mycel-proto/build.rs` discovers all `*.proto` files under the `mycel-api` checkout and generates Rust code into Cargo's build output during `cargo build`/`cargo test`.

By default, it reads:

```text
../mycel-api/api/proto
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

Admin backup helpers wrap `mycel.admin.v1.AdminBackupService`:

```rust
let policy = admin.get_backup_policy().await?;
let status = admin.get_backup_status().await?;
let trigger = admin.trigger_backup("before upgrade").await?;
let _ = (policy, status, trigger);
```

## Environment config

`Config::from_env()` reads:

- `MYCELD_GRPC_ADDR`
- `MYCEL_USERNAME`
- `MYCEL_PASSWORD`
- `MYCEL_ACCESS_TOKEN`
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
