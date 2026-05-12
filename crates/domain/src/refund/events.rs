use serde::{Deserialize, Serialize};

use crate::shared::domain_event::DomainEvent;
use crate::booking::value_objects::BookingId;
use super::value_objects::RefundId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequested {
    pub refund_id: RefundId,
    pub booking_id: BookingId,
}

impl DomainEvent for RefundRequested {
    fn event_name(&self) -> &'static str {
        "RefundRequested"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundApproved {
    pub refund_id: RefundId,
}

impl DomainEvent for RefundApproved {
    fn event_name(&self) -> &'static str {
        "RefundApproved"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRejected {
    pub refund_id: RefundId,
    pub reason: String,
}

impl DomainEvent for RefundRejected {
    fn event_name(&self) -> &'static str {
        "RefundRejected"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundProcessed {
    pub refund_id: RefundId,
}

impl DomainEvent for RefundProcessed {
    fn event_name(&self) -> &'static str {
        "RefundProcessed"
    }
}
