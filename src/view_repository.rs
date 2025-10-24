use std::marker::PhantomData;

use async_trait::async_trait;
use cqrs_es::persist::{PersistenceError, ViewContext, ViewRepository};
use cqrs_es::{Aggregate, View};
use sqlx::sqlite::SqliteRow;
use sqlx::{Pool, Row, Sqlite};

use crate::error::SqliteAggregateError;

/// A sqlite backed query repository for use in backing a `GenericQuery`.
pub struct SqliteViewRepository<V, A> {
    insert_sql: String,
    update_sql: String,
    select_sql: String,
    pool: Pool<Sqlite>,
    _phantom: PhantomData<(V, A)>,
}

impl<V, A> SqliteViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `SqliteViewRepository` that will store serialized views in a Sqlite table named
    /// identically to the `view_name` value provided. This table should be created by the user
    /// before using this query repository (see `/db/init.sql` sql initialization file).
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::persist::doc::MyView;
    /// use sqlx::{Pool, Sqlite};
    /// use sqlite_es::SqliteViewRepository;
    ///
    /// fn configure_view_repo(pool: Pool<Sqlite>) -> SqliteViewRepository<MyView,MyAggregate> {
    ///     SqliteViewRepository::new("my_view_table", pool)
    /// }
    /// ```
    pub fn new(view_name: &str, pool: Pool<Sqlite>) -> Self {
        let insert_sql = format!(
            "INSERT INTO {} (payload, version, view_id) VALUES ( $1, $2, $3 )",
            view_name
        );
        let update_sql = format!(
            "UPDATE {} SET payload= $1 , version= $2 WHERE view_id= $3",
            view_name
        );
        let select_sql = format!(
            "SELECT version,payload FROM {} WHERE view_id= $1",
            view_name
        );
        Self {
            insert_sql,
            update_sql,
            select_sql,
            pool,
            _phantom: Default::default(),
        }
    }
}

#[async_trait]
impl<V, A> ViewRepository<V, A> for SqliteViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError> {
        let row: Option<SqliteRow> = sqlx::query(&self.select_sql)
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(SqliteAggregateError::from)?;
        match row {
            None => Ok(None),
            Some(row) => {
                let view = serde_json::from_value(row.get("payload"))?;
                Ok(Some(view))
            }
        }
    }

    async fn load_with_context(
        &self,
        view_id: &str,
    ) -> Result<Option<(V, ViewContext)>, PersistenceError> {
        let row: Option<SqliteRow> = sqlx::query(&self.select_sql)
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(SqliteAggregateError::from)?;
        match row {
            None => Ok(None),
            Some(row) => {
                let version = row.get("version");
                let view = serde_json::from_value(row.get("payload"))?;
                let view_context = ViewContext::new(view_id.to_string(), version);
                Ok(Some((view, view_context)))
            }
        }
    }

    async fn update_view(&self, view: V, context: ViewContext) -> Result<(), PersistenceError> {
        let sql = match context.version {
            0 => &self.insert_sql,
            _ => &self.update_sql,
        };
        let version = context.version + 1;
        let payload = serde_json::to_value(&view).map_err(SqliteAggregateError::from)?;
        sqlx::query(sql.as_str())
            .bind(payload)
            .bind(version)
            .bind(context.view_instance_id)
            .execute(&self.pool)
            .await
            .map_err(SqliteAggregateError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        Created, TEST_CONNECTION_STRING, TestAggregate, TestEvent, TestView,
    };
    use crate::{SqliteViewRepository, default_sqlite_pool, init_tables};
    use cqrs_es::persist::{ViewContext, ViewRepository};

    #[tokio::test]
    async fn test_valid_view_repository() {
        let pool = default_sqlite_pool(TEST_CONNECTION_STRING).await;
        // we can use a tranditional wrapper around the `query!()` macro using files

        init_tables(&pool).await.unwrap();

        let repo = SqliteViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
        let test_view_id = uuid::Uuid::new_v4().to_string();

        let view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "just a test event for this view".to_string(),
            })],
        };
        repo.update_view(view.clone(), ViewContext::new(test_view_id.to_string(), 0))
            .await
            .unwrap();
        let (found, context) = repo
            .load_with_context(&test_view_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found, view);
        let found = repo.load(&test_view_id).await.unwrap().unwrap();
        assert_eq!(found, view);

        let updated_view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "a totally different view".to_string(),
            })],
        };
        repo.update_view(updated_view.clone(), context)
            .await
            .unwrap();
        let found_option = repo.load(&test_view_id).await.unwrap();
        let found = found_option.unwrap();

        assert_eq!(found, updated_view);
    }
}
