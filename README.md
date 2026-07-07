# Mycel Rust SDK

Rust connector for MycelDB daemon APIs.

This SDK mirrors the Go SDK shape:

- daemon dial helpers
- plaintext/TLS/mTLS transport config
- user login and refresh helpers
- operator/admin login helper
- bearer-token metadata injection
- generated Admin and Client service clients
- call timeout helpers
- session/transaction helpers
- thin graph/query convenience methods
- Admin backup policy/status/list/trigger/delete helpers

## Crates

- `mycel-proto`: generated `prost`/`tonic` protobuf and gRPC clients from `../mycel-api/api/proto`
- `mycel-sdk`: ergonomic client wrapper around the generated clients

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
