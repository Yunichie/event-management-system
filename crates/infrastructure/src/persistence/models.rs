use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "event_status", rename_all = "snake_case")]
pub enum EventStatusDb {
    Draft,
    Published,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "booking_status", rename_all = "snake_case")]
pub enum BookingStatusDb {
    PendingPayment,
    Paid,
    Expired,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")]
pub enum TicketStatusDb {
    Active,
    CheckedIn,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "refund_status", rename_all = "snake_case")]
pub enum RefundStatusDb {
    Requested,
    Approved,
    Rejected,
    PaidOut,
}


#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EventRow {
    pub id: Uuid,
    pub organizer_id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub location: String,
    pub max_capacity: i32,
    pub status: EventStatusDb,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TicketCategoryRow {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub price_amount: Decimal,
    pub price_currency: String,
    pub quota: i32,
    pub remaining_quota: i32,
    pub sales_start: NaiveDate,
    pub sales_end: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookingRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub event_id: Uuid,
    pub category_id: Uuid,
    pub quantity: i32,
    pub total_amount: Decimal,
    pub total_currency: String,
    pub status: BookingStatusDb,
    pub payment_deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TicketRow {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub event_id: Uuid,
    pub code: String,
    pub status: TicketStatusDb,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TicketWithCategoryRow {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub event_id: Uuid,
    pub code: String,
    pub status: TicketStatusDb,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefundRow {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub customer_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub status: RefundStatusDb,
    pub rejection_reason: Option<String>,
    pub payment_reference: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
