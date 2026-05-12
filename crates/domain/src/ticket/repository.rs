use async_trait::async_trait;

use crate::shared::errors::RepoError;

use super::{
    entity::Ticket,
    value_objects::{TicketCode, TicketId},
};

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn save(&self, ticket: &mut Ticket) -> Result<(), RepoError>;
    async fn save_multiple(&self, tickets: &mut [Ticket]) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: TicketId) -> Result<Option<Ticket>, RepoError>;
    async fn find_by_code(&self, code: &TicketCode) -> Result<Option<Ticket>, RepoError>;
}