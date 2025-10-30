# sqlite-es

A Sqlite implementation of the `PersistedEventRepository` trait in [cqrs-es](https://crates.io/crates/cqrs-es).

---

## Usage
Add to your Cargo.toml file:

```toml
[dependencies]
cqrs-es = "0.4.11"
sqlite-es = "0.1.0"
```

Requires access to a Sqlite DB with existing tables, see: [Sample database configuration](db/init.sql).

Sample setup:
```rust
use sqlite_es::{
    default_sqlite_pool, init_tables, SqliteCqrs, sqlite_cqrs,
    tests::{TestAggregate, TestServices},
};

async fn setup_cqrs() -> SqliteCqrs<TestAggregate> {
    let pool = default_sqlite_pool("sqlite::memory:").await; // or 'sqlite:file.db?mode=rwc' for file
    init_tables(&pool)
        .await
        .expect("initialize db tables");
    sqlite_cqrs::<TestAggregate>(pool, vec![], TestServices {})
}
```

Advandced pool setup:
```rust
use std::str::FromStr;

use cqrs_es::EventStore;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};

use sqlite_es::{
    init_tables, SqliteCqrs, sqlite_aggregate_cqrs,
    tests::{TestAggregate, TestServices},
};

async fn setup_cqrs<ES: EventStore<TestAggregate>>() -> SqliteCqrs<TestAggregate> {
    let opts = SqliteConnectOptions::from_str("sqlite:file.db?mode=rwc")
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    let pool = SqlitePool::connect_with(opts).await.expect("connect pool");

    init_tables(&pool)
        .await
        .expect("initialize db tables");

    sqlite_aggregate_cqrs::<TestAggregate>(pool, vec![], TestServices {})
}
```

## Runtime and TLS configuration
This package defaults to expect the [Tokio runtime](https://crates.io/crates/tokio) and the
[Rustls library](https://crates.io/crates/rustls) for TLS.
If a different combination is desired the appropriate feature flag should be used:
- `runtime-tokio-native-tls`
- `runtime-tokio-rustls` (default)
- `runtime-async-std-native-tls`
- `runtime-async-std-rustls`
- `runtime-actix-native-tls`
- `runtime-actix-rustls`

[![Crates.io](https://img.shields.io/crates/v/sqlite-es-mk)](https://crates.io/crates/sqlite-es-mk)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/sqlite-es-mk)
![docs](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiVVUyR0tRbTZmejFBYURoTHdpR3FnSUFqKzFVZE9JNW5haDZhcUFlY2xtREhtaVVJMWsxcWZOeC8zSUR0UWhpaWZMa0ZQSHlEYjg0N2FoU2lwV1FsTXFRPSIsIml2UGFyYW1ldGVyU3BlYyI6IldjUVMzVEpKN1V3aWxXWGUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=main)
