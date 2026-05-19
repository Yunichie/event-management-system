use uuid::Uuid;
use rust_decimal::Decimal;

pub struct GetEventQuery {
    pub event_id: Uuid,
}

pub struct GetPublishedEventsQuery {
    pub date: Option<chrono::NaiveDate>,
    pub location: Option<String>,
}

pub struct CalculateBookingPriceQuery {
    pub event_id: Uuid,
    pub category_id: Uuid,
    pub quantity: u32,
}

pub struct GetEventSalesReportQuery {
    pub event_id: Uuid,
}

pub struct GetEventParticipantsQuery {
    pub event_id: Uuid,
}
