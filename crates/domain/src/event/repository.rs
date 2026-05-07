use async_trait::async_trait;

use crate::shared::errors::RepoError;

use super::{aggregate::Event, value_objects::EventId};

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &mut Event) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: EventId) -> Result<Option<Event>, RepoError>;
}
