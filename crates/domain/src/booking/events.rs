use serde::{Deserialize, Serialize};

use crate::shared::{domain_event::DomainEvent, value_objects::UserId};
use crate::event::value_objects::{CategoryId, EventId};
use super::value_objects::BookingId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingCreated {
    pub booking_id: BookingId,
    pub user_id: UserId,
    pub event_id: EventId,
}

impl DomainEvent for BookingCreated {
    fn event_name(&self) -> &'static str {
        "BookingCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketReserved {
    pub booking_id: BookingId,
    pub event_id: EventId,
    pub quantity: u32,
}

impl DomainEvent for TicketReserved {
    fn event_name(&self) -> &'static str {
        "TicketReserved"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingPaid {
    pub booking_id: BookingId,
}

impl DomainEvent for BookingPaid {
    fn event_name(&self) -> &'static str {
        "BookingPaid"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingExpired {
    pub booking_id: BookingId,
    pub category_id: CategoryId,
    pub quantity: u32,
}

impl DomainEvent for BookingExpired {
    fn event_name(&self) -> &'static str {
        "BookingExpired"
    }
}
