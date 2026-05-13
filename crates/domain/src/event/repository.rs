use async_trait::async_trait;

use crate::shared::errors::RepoError;

use super::{aggregate::Event, value_objects::EventId};

#[derive(Debug, Default)]
pub struct EventFilter {
    pub date: Option<chrono::NaiveDate>,
    pub location: Option<String>,
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &mut Event) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: EventId) -> Result<Option<Event>, RepoError>;
    async fn find_published(&self, filter: EventFilter) -> Result<Vec<Event>, RepoError>;
}
