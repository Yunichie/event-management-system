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
pub struct EventPublished {
    pub event_id: EventId,
}

impl DomainEvent for EventPublished {
    fn event_name(&self) -> &'static str {
        "EventPublished"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCancelled {
    pub event_id: EventId,
}

impl DomainEvent for EventCancelled {
    fn event_name(&self) -> &'static str {
        "EventCancelled"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCategoryCreated {
    pub event_id: EventId,
    pub category_id: CategoryId,
    pub name: String,
}

impl DomainEvent for TicketCategoryCreated {
    fn event_name(&self) -> &'static str {
        "TicketCategoryCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCategoryDisabled {
    pub event_id: EventId,
    pub category_id: CategoryId,
}

impl DomainEvent for TicketCategoryDisabled {
    fn event_name(&self) -> &'static str {
        "TicketCategoryDisabled"
    }
}
