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
    pub checked_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Ticket {
    pub fn issue(
        booking_id: BookingId,
        event_id: EventId,
        category_id: CategoryId,
    ) -> Self {
        let id = TicketId::new();
        let code = TicketCode::generate();

        let mut ticket = Self {
            id,
            booking_id,
            event_id,
            category_id,
            code: code.clone(),
            status: TicketStatus::Active,
            checked_in_at: None,
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

    pub fn check_in(&mut self, event_id: EventId, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.event_id != event_id {
            return Err(DomainError::TicketEventMismatch);
        }

        if self.status == TicketStatus::CheckedIn {
            return Err(DomainError::AlreadyCheckedIn);
        }

        if self.status != TicketStatus::Active {
            return Err(DomainError::InvalidStatusTransition(
                "Ticket is not active for check-in".to_string(),
            ));
        }

        self.status = TicketStatus::CheckedIn;
        self.checked_in_at = Some(now);

        self.events.push(Box::new(TicketCheckedIn {
            ticket_id: self.id,
            event_id: self.event_id,
        }));

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status == TicketStatus::CheckedIn {
            return Err(DomainError::CheckedInTicketRefundDenied);
        }

        if self.status == TicketStatus::Cancelled {
            return Err(DomainError::InvalidStatusTransition(
                "Ticket is already cancelled".to_string(),
            ));
        }

        self.status = TicketStatus::Cancelled;
        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<DomainEventBox> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ticket() -> Ticket {
        Ticket::issue(
            BookingId::new(),
            EventId::new(),
            CategoryId::new(),
        )
    }

    #[test]
    fn check_in_sets_status_to_checked_in() {
        let mut ticket = make_ticket();
        let event_id = ticket.event_id;
        let now = Utc::now();

        ticket.check_in(event_id, now).unwrap();

        assert_eq!(ticket.status, TicketStatus::CheckedIn);
        assert!(ticket.checked_in_at.is_some());
    }

    #[test]
    fn check_in_fails_if_already_checked_in() {
        let mut ticket = make_ticket();
        let event_id = ticket.event_id;
        let now = Utc::now();

        ticket.check_in(event_id, now).unwrap();
        let result = ticket.check_in(event_id, now);

        assert!(matches!(result, Err(DomainError::AlreadyCheckedIn)));
    }

    #[test]
    fn check_in_fails_if_event_id_does_not_match() {
        let mut ticket = make_ticket();
        let wrong_event_id = EventId::new();
        let now = Utc::now();

        let result = ticket.check_in(wrong_event_id, now);

        assert!(matches!(result, Err(DomainError::TicketEventMismatch)));
    }

    #[test]
    fn cancel_sets_status_to_cancelled() {
        let mut ticket = make_ticket();
        ticket.cancel().unwrap();
        assert_eq!(ticket.status, TicketStatus::Cancelled);
    }

    #[test]
    fn cancel_fails_if_already_checked_in() {
        let mut ticket = make_ticket();
        let event_id = ticket.event_id;
        ticket.check_in(event_id, Utc::now()).unwrap();

        let result = ticket.cancel();
        assert!(matches!(result, Err(DomainError::CheckedInTicketRefundDenied)));
    }
}
