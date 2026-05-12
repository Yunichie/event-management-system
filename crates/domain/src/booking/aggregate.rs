use chrono::{DateTime, Duration, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::{Money, UserId},
};
use crate::event::value_objects::{CategoryId, EventId};

use super::{
    events::{BookingCancelled, BookingCreated, BookingPaid},
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
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
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
        let expires_at = now + Duration::minutes(15); // e.g. 15 minutes to pay

        let mut booking = Self {
            id,
            user_id,
            event_id,
            category_id,
            quantity,
            total_amount,
            status: BookingStatus::Pending,
            created_at: now,
            expires_at,
            events: Vec::new(),
        };

        booking.events.push(Box::new(BookingCreated {
            booking_id: id,
            user_id,
            event_id,
        }));

        Ok(booking)
    }

    pub fn pay(&mut self, amount: Money) -> Result<(), DomainError> {
        if self.status != BookingStatus::Pending {
            return Err(DomainError::InvalidStatusTransition(
                "Can only pay for pending bookings".to_string(),
            ));
        }

        if Utc::now() > self.expires_at {
            return Err(DomainError::PaymentDeadlinePassed);
        }

        if self.total_amount.currency() != amount.currency()
            || self.total_amount.amount() != amount.amount()
        {
            return Err(DomainError::PaymentAmountMismatch);
        }

        self.status = BookingStatus::Paid;

        self.events.push(Box::new(BookingPaid {
            booking_id: self.id,
        }));

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status != BookingStatus::Pending {
            return Err(DomainError::InvalidStatusTransition(
                "Can only cancel pending bookings".to_string(),
            ));
        }

        self.status = BookingStatus::Cancelled;

        self.events.push(Box::new(BookingCancelled {
            booking_id: self.id,
        }));

        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<DomainEventBox> {
        std::mem::take(&mut self.events)
    }
}
