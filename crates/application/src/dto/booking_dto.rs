use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub category_id: Uuid,
    pub quantity: u32,
    pub total_amount: Decimal,
    pub currency: String,
    pub status: String,
    pub payment_deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
