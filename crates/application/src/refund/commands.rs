use uuid::Uuid;

pub struct RequestRefundCommand {
    pub booking_id: Uuid,
    pub user_id: Uuid,
    pub reason: Option<String>,
}

pub struct ApproveRefundCommand {
    pub refund_id: Uuid,
}

pub struct RejectRefundCommand {
    pub refund_id: Uuid,
    pub rejection_reason: String,
}

pub struct PayoutRefundCommand {
    pub refund_id: Uuid,
}
