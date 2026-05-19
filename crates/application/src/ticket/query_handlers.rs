use std::sync::Arc;

use domain::ticket::{
    entity::Ticket, repository::TicketRepository, value_objects::{TicketCode, TicketId},
};

use crate::dto::TicketDto;
use crate::errors::{ApplicationError, AppResult};

use super::queries::{GetTicketByCodeQuery, GetTicketQuery};

pub struct TicketQueryHandler<R: TicketRepository> {
    ticket_repo: Arc<R>,
}

impl<R: TicketRepository> TicketQueryHandler<R> {
    pub fn new(ticket_repo: Arc<R>) -> Self {
        Self { ticket_repo }
    }

    pub async fn handle_get_ticket(&self, query: GetTicketQuery) -> AppResult<TicketDto> {
        let ticket_id = TicketId::from(query.ticket_id);
        let ticket = self
            .ticket_repo
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Ticket not found".to_string()))?;

        Ok(self.ticket_to_dto(&ticket))
    }

    pub async fn handle_get_ticket_by_code(
        &self,
        query: GetTicketByCodeQuery,
    ) -> AppResult<TicketDto> {
        let ticket_code = TicketCode::new(query.ticket_code);
        let ticket = self
            .ticket_repo
            .find_by_code(&ticket_code)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Ticket not found".to_string()))?;

        Ok(self.ticket_to_dto(&ticket))
    }

    fn ticket_to_dto(&self, ticket: &Ticket) -> TicketDto {
        TicketDto {
            id: ticket.id.into_inner(),
            booking_id: ticket.booking_id.into_inner(),
            event_id: ticket.event_id.into_inner(),
            category_id: ticket.category_id.into_inner(),
            code: ticket.code.as_str().to_string(),
            status: format!("{}", ticket.status),
            checked_in_at: ticket.checked_in_at,
            created_at: ticket.created_at,
        }
    }
}
