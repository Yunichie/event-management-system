use std::sync::Arc;

use chrono::Utc;
use domain::event::value_objects::EventId;
use domain::ticket::{repository::TicketRepository, value_objects::TicketCode};

use crate::dto::TicketDto;
use crate::errors::{ApplicationError, AppResult};

use super::commands::CheckInTicketCommand;

pub struct TicketCommandHandler<R: TicketRepository> {
    ticket_repo: Arc<R>,
}

impl<R: TicketRepository> TicketCommandHandler<R> {
    pub fn new(ticket_repo: Arc<R>) -> Self {
        Self { ticket_repo }
    }

    pub async fn handle_check_in_ticket(&self, cmd: CheckInTicketCommand) -> AppResult<TicketDto> {
        let ticket_code = TicketCode::new(cmd.ticket_code);
        let event_id = EventId::from(cmd.event_id);

        let mut ticket = self
            .ticket_repo
            .find_by_code(&ticket_code)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Ticket not found".to_string()))?;

        let now = Utc::now();
        ticket.check_in(event_id, now)?;

        self.ticket_repo.save(&mut ticket).await?;

        Ok(TicketDto {
            id: ticket.id.into_inner(),
            booking_id: ticket.booking_id.into_inner(),
            event_id: ticket.event_id.into_inner(),
            category_id: ticket.category_id.into_inner(),
            code: ticket.code.as_str().to_string(),
            status: format!("{}", ticket.status),
            checked_in_at: ticket.checked_in_at,
            created_at: ticket.created_at,
        })
    }
}
