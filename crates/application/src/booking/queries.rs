use uuid::Uuid;

pub struct GetBookingQuery {
    pub booking_id: Uuid,
}

pub struct GetCustomerBookingsQuery {
    pub customer_id: Uuid,
    pub event_id: Uuid,
}
