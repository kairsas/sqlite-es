use crate::SqliteEventRepository;
use cqrs_es::CqrsFramework;
use cqrs_es::persist::PersistedEventStore;

/// A convenience type for a CqrsFramework backed by
/// [RusqliteStore](struct.RusqliteStore.html).
pub type SqliteCqrs<A> = CqrsFramework<A, PersistedEventStore<SqliteEventRepository, A>>;
