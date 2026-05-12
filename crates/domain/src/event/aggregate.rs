use chrono::{DateTime, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::Money,
};

use super::{
    events::{EventCreated, EventPublished, TicketCategoryAdded},
    ticket_category::TicketCategory,
    value_objects::{CategoryId, EventId, EventStatus},
};

pub struct Event {
    pub id: EventId,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub capacity: u32,
    pub status: EventStatus,
    pub categories: Vec<TicketCategory>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Event {
    pub fn new(
        id: EventId,
        name: String,
        description: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        capacity: u32,
    ) -> Result<Self, DomainError> {
        if end_date <= start_date {
            return Err(DomainError::InvalidSchedule);
        }
        if capacity == 0 {
            return Err(DomainError::InvalidCapacity);
        }

        let now = Utc::now();
        let mut event = Self {
            id,
            name: name.clone(),
            description,
            start_date,
            end_date,
            capacity,
            status: EventStatus::Draft,
            categories: Vec::new(),
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        event.events.push(Box::new(EventCreated {
            event_id: id,
            name,
        }));

        Ok(event)
    }

    pub fn add_ticket_category(
        &mut self,
        category_id: CategoryId,
        name: String,
        price: Money,
        quota: u32,
        sales_start: DateTime<Utc>,
        sales_end: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if self.status != EventStatus::Draft {
            return Err(DomainError::InvalidStatusTransition(
                "Can only add categories to draft events".to_string(),
            ));
        }

        if quota == 0 {
            return Err(DomainError::InvalidQuota);
        }

        if sales_end > self.start_date {
            return Err(DomainError::InvalidSalesPeriod);
        }

        let total_quota: u32 = self.categories.iter().map(|c| c.quota).sum();
        if total_quota + quota > self.capacity {
            return Err(DomainError::QuotaExceedsCapacity);
        }

        let category = TicketCategory::new(
            category_id,
            self.id,
            name.clone(),
            price,
            quota,
            sales_start,
            sales_end,
        );

        self.categories.push(category);
        self.events.push(Box::new(TicketCategoryAdded {
            event_id: self.id,
            category_id,
            name,
        }));

        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        if self.status != EventStatus::Draft {
            return Err(DomainError::InvalidStatusTransition(
                "Can only publish draft events".to_string(),
            ));
        }

        if self.categories.is_empty() {
            return Err(DomainError::BusinessRule(
                "Cannot publish event without ticket categories".to_string(),
            ));
        }

        self.status = EventStatus::Published;
        self.updated_at = Utc::now();

        self.events.push(Box::new(EventPublished {
            event_id: self.id,
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
    use chrono::Duration;
    use rust_decimal_macros::dec;

    #[test]
    fn create_event_success() {
        let now = Utc::now();
        let event = Event::new(
            EventId::new(),
            "Tech Conf".to_string(),
            "A cool conf".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            1000,
        )
        .unwrap();

        assert_eq!(event.name, "Tech Conf");
        assert_eq!(event.capacity, 1000);
        assert_eq!(event.status, EventStatus::Draft);
        assert_eq!(event.events.len(), 1);
    }

    #[test]
    fn create_event_fails_invalid_schedule() {
        let now = Utc::now();
        let result = Event::new(
            EventId::new(),
            "Tech Conf".to_string(),
            "".to_string(),
            now + Duration::days(12),
            now + Duration::days(10),
            1000,
        );

        assert!(matches!(result, Err(DomainError::InvalidSchedule)));
    }

    #[test]
    fn add_ticket_category_success() {
        let now = Utc::now();
        let mut event = Event::new(
            EventId::new(),
            "Tech Conf".to_string(),
            "".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            100,
        )
        .unwrap();

        event.take_events(); // clear creation events

        event
            .add_ticket_category(
                CategoryId::new(),
                "Early Bird".to_string(),
                Money::new(dec!(50.0), "USD").unwrap(),
                50,
                now,
                now + Duration::days(5),
            )
            .unwrap();

        assert_eq!(event.categories.len(), 1);
        assert_eq!(event.events.len(), 1); // category added event
    }

    #[test]
    fn add_ticket_category_fails_exceeds_capacity() {
        let now = Utc::now();
        let mut event = Event::new(
            EventId::new(),
            "Tech Conf".to_string(),
            "".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            100, // Capacity is 100
        )
        .unwrap();

        let result = event.add_ticket_category(
            CategoryId::new(),
            "VIP".to_string(),
            Money::new(dec!(50.0), "USD").unwrap(),
            150, // Requesting 150 quota
            now,
            now + Duration::days(5),
        );

        assert!(matches!(result, Err(DomainError::QuotaExceedsCapacity)));
    }
}
