use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};

use crate::{SqliteCqrs, SqliteEventRepository};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

/// A convenience method for building a simple connection pool for SqliteDb.
/// A connection pool is needed for both the event and view repositories.
/// Use `sqlite:file.db?mode=rwc` for auto creating a file.
/// Use empty string for temp file.
/// Use `sqlite::memory:` for in-memory.
///
/// ```
/// use sqlx::{Pool, Sqlite};
/// use sqlite_es::default_sqlite_pool;
///
/// # async fn configure_pool() {
/// let connection_string = "sqlite::memory:";
/// let pool: Pool<Sqlite> = default_sqlite_pool(connection_string).await;
/// # }
/// ```
pub async fn default_sqlite_pool(connection_string: &str) -> Pool<Sqlite> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(connection_string)
        .await
        .expect("unable to connect to database")
}

/// A convenience function for creating a CqrsFramework from a database connection pool
/// and queries.
pub fn sqlite_cqrs<A>(
    pool: Pool<Sqlite>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> SqliteCqrs<A>
where
    A: Aggregate,
{
    let repo = SqliteEventRepository::new(pool);
    let store = PersistedEventStore::new_event_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using a snapshot store.
pub fn sqlite_snapshot_cqrs<A>(
    pool: Pool<Sqlite>,
    query_processor: Vec<Box<dyn Query<A>>>,
    snapshot_size: usize,
    services: A::Services,
) -> SqliteCqrs<A>
where
    A: Aggregate,
{
    let repo = SqliteEventRepository::new(pool);
    let store = PersistedEventStore::new_snapshot_store(repo, snapshot_size);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using an aggregate store.
pub fn sqlite_aggregate_cqrs<A>(
    pool: Pool<Sqlite>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> SqliteCqrs<A>
where
    A: Aggregate,
{
    let repo = SqliteEventRepository::new(pool);
    let store = PersistedEventStore::new_aggregate_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        TEST_CONNECTION_STRING, TestAggregate, TestQueryRepository, TestServices, TestView,
    };
    use crate::{SqliteViewRepository, default_sqlite_pool, sqlite_cqrs};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let pool = default_sqlite_pool(TEST_CONNECTION_STRING).await;
        let repo = SqliteViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
        let query = TestQueryRepository::new(Arc::new(repo));
        let _ps = sqlite_cqrs(pool, vec![Box::new(query)], TestServices);
    }
}
