use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct CreateEventCommand {
    pub organizer_id: Uuid,
    pub name: String,
    pub description: String,
    pub location: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub max_capacity: u32,
}

pub struct PublishEventCommand {
    pub event_id: Uuid,
}

pub struct CancelEventCommand {
    pub event_id: Uuid,
}

pub struct AddTicketCategoryCommand {
    pub event_id: Uuid,
    pub name: String,
    pub price: Decimal,
    pub currency: String,
    pub quota: u32,
    pub sales_start: DateTime<Utc>,
    pub sales_end: DateTime<Utc>,
}

pub struct DisableTicketCategoryCommand {
    pub event_id: Uuid,
    pub category_id: Uuid,
}
