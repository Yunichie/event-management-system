use uuid::Uuid;

pub struct GetRefundQuery {
    pub refund_id: Uuid,
}

pub struct GetRefundByBookingQuery {
    pub booking_id: Uuid,
}
