use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDto {
    pub id: Uuid,
    pub organizer_id: Uuid,
    pub name: String,
    pub description: String,
    pub location: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub max_capacity: u32,
    pub status: String,
    pub categories: Vec<TicketCategoryDto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCategoryDto {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub price: Decimal,
    pub currency: String,
    pub quota: u32,
    pub remaining_quota: u32,
    pub sales_start: DateTime<Utc>,
    pub sales_end: DateTime<Utc>,
    pub is_active: bool,
}
