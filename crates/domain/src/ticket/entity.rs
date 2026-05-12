use chrono::{DateTime, Utc};

use crate::shared::{domain_event::DomainEventBox, errors::DomainError};
use crate::event::value_objects::{CategoryId, EventId};
use crate::booking::value_objects::BookingId;

use super::{
    events::{TicketCheckedIn, TicketIssued},
    value_objects::{TicketCode, TicketId, TicketStatus},
};

pub struct Ticket {
    pub id: TicketId,
    pub booking_id: BookingId,
    pub event_id: EventId,
    pub category_id: CategoryId,
    pub code: TicketCode,
    pub status: TicketStatus,
    pub created_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Ticket {
    pub fn issue(
        id: TicketId,
        booking_id: BookingId,
        event_id: EventId,
        category_id: CategoryId,
        code: TicketCode,
    ) -> Self {
        let mut ticket = Self {
            id,
            booking_id,
            event_id,
            category_id,
            code: code.clone(),
            status: TicketStatus::Valid,
            created_at: Utc::now(),
            events: Vec::new(),
        };

        ticket.events.push(Box::new(TicketIssued {
            ticket_id: id,
            booking_id,
            event_id,
            code,
        }));

        ticket
    }

    pub fn check_in(&mut self, event_id: EventId) -> Result<(), DomainError> {
        if self.event_id != event_id {
            return Err(DomainError::TicketEventMismatch);
        }

        if self.status == TicketStatus::CheckedIn {
            return Err(DomainError::AlreadyCheckedIn);
        }

        if self.status != TicketStatus::Valid {
            return Err(DomainError::InvalidStatusTransition(
                "Ticket is not valid for check-in".to_string(),
            ));
        }

        self.status = TicketStatus::CheckedIn;

        self.events.push(Box::new(TicketCheckedIn {
            ticket_id: self.id,
            event_id: self.event_id,
        }));

        Ok(())
    }

    pub fn refund(&mut self) -> Result<(), DomainError> {
        if self.status == TicketStatus::CheckedIn {
            return Err(DomainError::CheckedInTicketRefundDenied);
        }

        if self.status == TicketStatus::Refunded {
            return Err(DomainError::InvalidStatusTransition(
                "Ticket is already refunded".to_string(),
            ));
        }

        self.status = TicketStatus::Refunded;
        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<DomainEventBox> {
        std::mem::take(&mut self.events)
    }
}
