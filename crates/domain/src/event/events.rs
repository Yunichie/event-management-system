use serde::{Deserialize, Serialize};

use crate::shared::domain_event::DomainEvent;

use super::value_objects::{CategoryId, EventId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCreated {
    pub event_id: EventId,
    pub name: String,
}

impl DomainEvent for EventCreated {
    fn event_name(&self) -> &'static str {
        "EventCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCategoryAdded {
    pub event_id: EventId,
    pub category_id: CategoryId,
    pub name: String,
}

impl DomainEvent for TicketCategoryAdded {
    fn event_name(&self) -> &'static str {
        "TicketCategoryAdded"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPublished {
    pub event_id: EventId,
}

impl DomainEvent for EventPublished {
    fn event_name(&self) -> &'static str {
        "EventPublished"
    }
}
