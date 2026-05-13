use chrono::{DateTime, Duration, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::{Money, UserId},
};
use crate::event::value_objects::{CategoryId, EventId};
use crate::ticket::entity::Ticket;

use super::{
    events::{BookingCreated, BookingExpired, BookingPaid, TicketReserved},
    value_objects::{BookingId, BookingStatus},
};

pub struct Booking {
    pub id: BookingId,
    pub user_id: UserId,
    pub event_id: EventId,
    pub category_id: CategoryId,
    pub quantity: u32,
    pub total_amount: Money,
    pub status: BookingStatus,
    pub payment_deadline: DateTime<Utc>,
    pub tickets: Vec<Ticket>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Booking {
    pub fn new(
        id: BookingId,
        user_id: UserId,
        event_id: EventId,
        category_id: CategoryId,
        quantity: u32,
        price_per_ticket: Money,
    ) -> Result<Self, DomainError> {
        if quantity == 0 {
            return Err(DomainError::BusinessRule(
                "Booking quantity must be greater than zero".to_string(),
            ));
        }

        let total_amount = price_per_ticket.multiply(quantity as i32)?;
        let now = Utc::now();
        let payment_deadline = now + Duration::minutes(15);

        let mut booking = Self {
            id,
            user_id,
            event_id,
            category_id,
            quantity,
            total_amount,
            status: BookingStatus::PendingPayment,
            payment_deadline,
            tickets: Vec::new(),
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        booking.events.push(Box::new(BookingCreated {
            booking_id: id,
            user_id,
            event_id,
        }));

        booking.events.push(Box::new(TicketReserved {
            booking_id: id,
            event_id,
            quantity,
        }));

        Ok(booking)
    }

    pub fn pay(&mut self, amount: &Money, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.status != BookingStatus::PendingPayment {
            return Err(DomainError::InvalidStatusTransition(
                "Can only pay for pending bookings".to_string(),
            ));
        }

        if now > self.payment_deadline {
            return Err(DomainError::PaymentDeadlinePassed);
        }

        if self.total_amount.currency() != amount.currency()
            || self.total_amount.amount() != amount.amount()
        {
            return Err(DomainError::PaymentAmountMismatch);
        }

        self.status = BookingStatus::Paid;
        self.updated_at = now;

        for _ in 0..self.quantity {
            let ticket = Ticket::issue(self.id, self.event_id, self.category_id);
            self.tickets.push(ticket);
        }

        self.events.push(Box::new(BookingPaid {
            booking_id: self.id,
        }));

        Ok(())
    }

    pub fn expire(&mut self) -> Result<(), DomainError> {
        if self.status != BookingStatus::PendingPayment {
            return Err(DomainError::InvalidStatusTransition(
                format!("Can only expire pending bookings, current status: {}", self.status),
            ));
        }

        self.status = BookingStatus::Expired;
        self.updated_at = Utc::now();

        self.events.push(Box::new(BookingExpired {
            booking_id: self.id,
            category_id: self.category_id,
            quantity: self.quantity,
        }));

        Ok(())
    }

    pub fn mark_refund_required(&mut self) -> Result<(), DomainError> {
        if self.status != BookingStatus::Paid {
            return Err(DomainError::InvalidStatusTransition(
                format!("Can only refund paid bookings, current status: {}", self.status),
            ));
        }

        self.status = BookingStatus::Refunded;
        self.updated_at = Utc::now();

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

    fn make_booking() -> Booking {
        Booking::new(
            BookingId::new(),
            UserId::new(),
            EventId::new(),
            CategoryId::new(),
            2,
            Money::new(dec!(100.0), "IDR").unwrap(),
        )
        .unwrap()
    }

    fn make_booking_with_deadline(deadline: DateTime<Utc>) -> Booking {
        let mut booking = make_booking();
        booking.payment_deadline = deadline;
        booking
    }

    #[test]
    fn create_booking_fails_with_zero_quantity() {
        let result = Booking::new(
            BookingId::new(),
            UserId::new(),
            EventId::new(),
            CategoryId::new(),
            0,
            Money::new(dec!(100.0), "IDR").unwrap(),
        );
        assert!(matches!(result, Err(DomainError::BusinessRule(_))));
    }

    #[test]
    fn pay_booking_fails_after_deadline() {
        let past_deadline = Utc::now() - Duration::minutes(1);
        let mut booking = make_booking_with_deadline(past_deadline);
        let amount = Money::new(dec!(200.0), "IDR").unwrap();

        let result = booking.pay(&amount, Utc::now());
        assert!(matches!(result, Err(DomainError::PaymentDeadlinePassed)));
    }

    #[test]
    fn pay_booking_fails_with_wrong_amount() {
        let mut booking = make_booking();
        let wrong_amount = Money::new(dec!(999.0), "IDR").unwrap();
        let now = Utc::now();

        let result = booking.pay(&wrong_amount, now);
        assert!(matches!(result, Err(DomainError::PaymentAmountMismatch)));
    }

    #[test]
    fn paid_booking_cannot_expire() {
        let mut booking = make_booking();
        let amount = booking.total_amount.clone();
        let now = Utc::now();
        booking.pay(&amount, now).unwrap();

        let result = booking.expire();
        assert!(matches!(result, Err(DomainError::InvalidStatusTransition(_))));
    }

    #[test]
    fn booking_creates_tickets_on_payment() {
        let mut booking = make_booking();
        let amount = booking.total_amount.clone();
        let now = Utc::now();
        assert!(booking.tickets.is_empty());

        booking.pay(&amount, now).unwrap();

        assert_eq!(booking.tickets.len(), 2); // quantity = 2
        // All ticket codes should be unique
        let codes: Vec<_> = booking.tickets.iter().map(|t| t.code.as_str().to_string()).collect();
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique_codes.len());
    }

    #[test]
    fn create_booking_success() {
        let booking = make_booking();
        assert_eq!(booking.status, BookingStatus::PendingPayment);
        assert_eq!(booking.quantity, 2);
        assert!(booking.tickets.is_empty());
        // Should have BookingCreated and TicketReserved events
        assert_eq!(booking.events.len(), 2);
    }

    #[test]
    fn expire_booking_success() {
        let mut booking = make_booking();
        booking.expire().unwrap();

        assert_eq!(booking.status, BookingStatus::Expired);
    }

    #[test]
    fn mark_refund_required_success() {
        let mut booking = make_booking();
        let amount = booking.total_amount.clone();
        booking.pay(&amount, Utc::now()).unwrap();

        booking.mark_refund_required().unwrap();
        assert_eq!(booking.status, BookingStatus::Refunded);
    }

    #[test]
    fn mark_refund_required_fails_if_not_paid() {
        let mut booking = make_booking();
        let result = booking.mark_refund_required();
        assert!(matches!(result, Err(DomainError::InvalidStatusTransition(_))));
    }
}
