use chrono::{DateTime, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::{Money, UserId},
};
use crate::booking::value_objects::BookingId;

use super::{
    events::{RefundApproved, RefundPaidOut, RefundRejected, RefundRequested},
    value_objects::{RefundId, RefundStatus},
};

pub struct Refund {
    pub id: RefundId,
    pub booking_id: BookingId,
    pub user_id: UserId,
    pub amount: Money,
    pub status: RefundStatus,
    pub reason: Option<String>,
    pub rejection_reason: Option<String>,
    pub payment_reference: Option<String>,
    pub requested_at: DateTime<Utc>,
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
            rejection_reason: None,
            payment_reference: None,
            requested_at: Utc::now(),
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
        self.rejection_reason = Some(reason.clone());

        self.events.push(Box::new(RefundRejected {
            refund_id: self.id,
            reason,
        }));

        Ok(())
    }

    pub fn mark_paid_out(&mut self, payment_reference: String) -> Result<(), DomainError> {
        if self.status != RefundStatus::Approved {
            return Err(DomainError::RefundNotApproved);
        }

        if payment_reference.trim().is_empty() {
            return Err(DomainError::BusinessRule(
                "Payment reference is required for payout".to_string(),
            ));
        }

        self.status = RefundStatus::PaidOut;
        self.payment_reference = Some(payment_reference.clone());

        self.events.push(Box::new(RefundPaidOut {
            refund_id: self.id,
            payment_reference,
        }));

        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<DomainEventBox> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_refund() -> Refund {
        Refund::request(
            RefundId::new(),
            BookingId::new(),
            UserId::new(),
            Money::new(dec!(200.0), "IDR").unwrap(),
            Some("Changed my mind".to_string()),
        )
    }

    #[test]
    fn approve_fails_if_not_in_requested_status() {
        let mut refund = make_refund();
        refund.approve().unwrap(); // Now Approved

        let result = refund.approve(); // Try again
        assert!(matches!(result, Err(DomainError::RefundNotRequested)));
    }

    #[test]
    fn reject_requires_non_empty_reason() {
        let mut refund = make_refund();

        let result = refund.reject("".to_string());
        assert!(matches!(result, Err(DomainError::RejectionReasonRequired)));

        let result = refund.reject("   ".to_string());
        assert!(matches!(result, Err(DomainError::RejectionReasonRequired)));
    }

    #[test]
    fn mark_paid_out_fails_if_not_approved() {
        let mut refund = make_refund(); // status = Requested

        let result = refund.mark_paid_out("REF-123".to_string());
        assert!(matches!(result, Err(DomainError::RefundNotApproved)));
    }

    #[test]
    fn paid_out_refund_is_immutable() {
        let mut refund = make_refund();
        refund.approve().unwrap();
        refund.mark_paid_out("REF-123".to_string()).unwrap();

        assert_eq!(refund.status, RefundStatus::PaidOut);

        // Cannot approve again
        let result = refund.approve();
        assert!(matches!(result, Err(DomainError::RefundNotRequested)));

        // Cannot reject
        let result = refund.reject("reason".to_string());
        assert!(matches!(result, Err(DomainError::RefundNotRequested)));

        // Cannot mark paid out again
        let result = refund.mark_paid_out("REF-456".to_string());
        assert!(matches!(result, Err(DomainError::RefundNotApproved)));
    }

    #[test]
    fn approve_and_payout_success() {
        let mut refund = make_refund();
        refund.approve().unwrap();
        assert_eq!(refund.status, RefundStatus::Approved);

        refund.mark_paid_out("BANK-REF-001".to_string()).unwrap();
        assert_eq!(refund.status, RefundStatus::PaidOut);
        assert_eq!(refund.payment_reference.as_deref(), Some("BANK-REF-001"));
    }

    #[test]
    fn reject_success() {
        let mut refund = make_refund();
        refund.reject("Policy violation".to_string()).unwrap();

        assert_eq!(refund.status, RefundStatus::Rejected);
        assert_eq!(refund.rejection_reason.as_deref(), Some("Policy violation"));
        // Original customer reason should still be preserved
        assert_eq!(refund.reason.as_deref(), Some("Changed my mind"));
    }

    #[test]
    fn mark_paid_out_fails_with_empty_reference() {
        let mut refund = make_refund();
        refund.approve().unwrap();

        let result = refund.mark_paid_out("".to_string());
        assert!(matches!(result, Err(DomainError::BusinessRule(_))));
    }
}
