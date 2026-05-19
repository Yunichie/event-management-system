use rust_decimal::Decimal;
use uuid::Uuid;

pub struct CreateBookingCommand {
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub category_id: Uuid,
    pub quantity: u32,
}

pub struct PayBookingCommand {
    pub booking_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
}

pub struct ExpireBookingCommand {
    pub booking_id: Uuid,
}
