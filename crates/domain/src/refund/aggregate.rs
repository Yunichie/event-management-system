use chrono::{DateTime, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::{Money, UserId},
};
use crate::booking::value_objects::BookingId;

use super::{
    events::{RefundApproved, RefundProcessed, RefundRejected, RefundRequested},
    value_objects::{RefundId, RefundStatus},
};

pub struct Refund {
    pub id: RefundId,
    pub booking_id: BookingId,
    pub user_id: UserId,
    pub amount: Money,
    pub status: RefundStatus,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Refund {
    pub fn request(
        id: RefundId,
        booking_id: BookingId,
        user_id: UserId,
        amount: Money,
        reason: Option<String>,
    ) -> Self {
        let mut refund = Self {
            id,
            booking_id,
            user_id,
            amount,
            status: RefundStatus::Requested,
            reason,
            created_at: Utc::now(),
            events: Vec::new(),
        };

        refund.events.push(Box::new(RefundRequested {
            refund_id: id,
            booking_id,
        }));

        refund
    }

    pub fn approve(&mut self) -> Result<(), DomainError> {
        if self.status != RefundStatus::Requested {
            return Err(DomainError::RefundNotRequested);
        }

        self.status = RefundStatus::Approved;

        self.events.push(Box::new(RefundApproved {
            refund_id: self.id,
        }));

        Ok(())
    }

    pub fn reject(&mut self, reason: String) -> Result<(), DomainError> {
        if self.status != RefundStatus::Requested {
            return Err(DomainError::RefundNotRequested);
        }

        if reason.trim().is_empty() {
            return Err(DomainError::RejectionReasonRequired);
        }

        self.status = RefundStatus::Rejected;
        self.reason = Some(reason.clone());

        self.events.push(Box::new(RefundRejected {
            refund_id: self.id,
            reason,
        }));

        Ok(())
    }

    pub fn process(&mut self) -> Result<(), DomainError> {
        if self.status != RefundStatus::Approved {
            return Err(DomainError::RefundNotApproved);
        }

        self.status = RefundStatus::Processed;

        self.events.push(Box::new(RefundProcessed {
            refund_id: self.id,
        }));

        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<DomainEventBox> {
        std::mem::take(&mut self.events)
    }
}
