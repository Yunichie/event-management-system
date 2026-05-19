use std::sync::Arc;

use chrono::Utc;
use domain::event::{value_objects::EventId, repository::EventRepository};
use domain::ticket::{repository::TicketRepository, value_objects::TicketCode};

use crate::dto::TicketDto;
use crate::errors::{ApplicationError, AppResult};

use super::commands::CheckInTicketCommand;

pub struct TicketCommandHandler<TR: TicketRepository, ER: EventRepository> {
    ticket_repo: Arc<TR>,
    event_repo: Arc<ER>,
}

impl<TR: TicketRepository, ER: EventRepository> TicketCommandHandler<TR, ER> {
    pub fn new(ticket_repo: Arc<TR>, event_repo: Arc<ER>) -> Self {
        Self { ticket_repo, event_repo }
    }

    pub async fn handle_check_in_ticket(&self, cmd: CheckInTicketCommand) -> AppResult<TicketDto> {
        let ticket_code = TicketCode::new(cmd.ticket_code);
        let event_id = EventId::from(cmd.event_id);

        // Find ticket - return specific error if not found
        let mut ticket = self
            .ticket_repo
            .find_by_code(&ticket_code)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Ticket not found".to_string()))?;

        // Check if ticket belongs to the correct event
        if ticket.event_id != event_id {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Ticket does not match the event".to_string(),
                ),
            ));
        }

        // Check if event has been cancelled
        let event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        if format!("{}", event.status) == "Cancelled" {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Event has been cancelled".to_string(),
                ),
            ));
        }

        // Check if ticket has already been checked in
        if format!("{}", ticket.status) == "CheckedIn" {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Ticket has already been used".to_string(),
                ),
            ));
        }

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
