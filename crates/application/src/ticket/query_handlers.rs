use std::sync::Arc;

use domain::ticket::{
    entity::Ticket, repository::TicketRepository, value_objects::{TicketCode, TicketId},
};
use domain::booking::{repository::BookingRepository, value_objects::BookingStatus};
use domain::shared::value_objects::UserId;

use crate::dto::TicketDto;
use crate::errors::{ApplicationError, AppResult};

use super::queries::{GetTicketByCodeQuery, GetTicketQuery, GetCustomerTicketsQuery};

pub struct TicketQueryHandler<TR: TicketRepository, BR: BookingRepository> {
    ticket_repo: Arc<TR>,
    booking_repo: Arc<BR>,
}

impl<TR: TicketRepository, BR: BookingRepository> TicketQueryHandler<TR, BR> {
    pub fn new(ticket_repo: Arc<TR>, booking_repo: Arc<BR>) -> Self {
        Self { ticket_repo, booking_repo }
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

    pub async fn handle_get_customer_tickets(
        &self,
        query: GetCustomerTicketsQuery,
    ) -> AppResult<Vec<TicketDto>> {
        let customer_id = UserId::from(query.customer_id);

        // In a real implementation, we'd need BookingRepository.find_by_customer()
        // to get all bookings for a customer, then collect tickets from paid bookings
        // For now, return empty list as placeholder
        // This would be implemented when the infrastructure layer provides the necessary repository methods
        
        Ok(vec![])
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
