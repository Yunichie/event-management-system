use serde::{Deserialize, Serialize};

use crate::shared::domain_event::DomainEvent;
use crate::event::value_objects::EventId;
use crate::booking::value_objects::BookingId;
use super::value_objects::{TicketId, TicketCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketIssued {
    pub ticket_id: TicketId,
    pub booking_id: BookingId,
    pub event_id: EventId,
    pub code: TicketCode,
}

impl DomainEvent for TicketIssued {
    fn event_name(&self) -> &'static str {
        "TicketIssued"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCheckedIn {
    pub ticket_id: TicketId,
    pub event_id: EventId,
}

impl DomainEvent for TicketCheckedIn {
    fn event_name(&self) -> &'static str {
        "TicketCheckedIn"
    }
}
