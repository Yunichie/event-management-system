use chrono::{DateTime, Utc};

use crate::shared::{
    domain_event::DomainEventBox,
    errors::DomainError,
    value_objects::{Money, UserId},
};

use super::{
    events::{EventCancelled, EventCreated, EventPublished, TicketCategoryCreated, TicketCategoryDisabled},
    ticket_category::TicketCategory,
    value_objects::{CategoryId, EventId, EventStatus},
};

pub struct Event {
    pub id: EventId,
    pub organizer_id: UserId,
    pub name: String,
    pub description: String,
    pub location: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub max_capacity: u32,
    pub status: EventStatus,
    pub categories: Vec<TicketCategory>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub events: Vec<DomainEventBox>,
}

impl Event {
    pub fn new(
        id: EventId,
        organizer_id: UserId,
        name: String,
        description: String,
        location: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        max_capacity: u32,
    ) -> Result<Self, DomainError> {
        if end_date <= start_date {
            return Err(DomainError::InvalidSchedule);
        }
        if max_capacity == 0 {
            return Err(DomainError::InvalidCapacity);
        }

        let now = Utc::now();
        let mut event = Self {
            id,
            organizer_id,
            name: name.clone(),
            description,
            location,
            start_date,
            end_date,
            max_capacity,
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

    #[allow(clippy::too_many_arguments)]
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
        if total_quota + quota > self.max_capacity {
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
        self.events.push(Box::new(TicketCategoryCreated {
            event_id: self.id,
            category_id,
            name,
        }));

        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        if self.status != EventStatus::Draft {
            return Err(DomainError::InvalidStatusTransition(
                format!("Can only publish draft events, current status: {}", self.status),
            ));
        }

        let has_active_category = self.categories.iter().any(|c| c.is_active);
        if !has_active_category {
            return Err(DomainError::BusinessRule(
                "Cannot publish event without at least one active ticket category".to_string(),
            ));
        }

        let total_quota: u32 = self.categories.iter()
            .filter(|c| c.is_active)
            .map(|c| c.quota)
            .sum();
        if total_quota > self.max_capacity {
            return Err(DomainError::QuotaExceedsCapacity);
        }

        self.status = EventStatus::Published;
        self.updated_at = Utc::now();

        self.events.push(Box::new(EventPublished {
            event_id: self.id,
        }));

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        match self.status {
            EventStatus::Published => {},
            EventStatus::Completed => {
                return Err(DomainError::InvalidStatusTransition(
                    "Cannot cancel a completed event".to_string(),
                ));
            }
            _ => {
                return Err(DomainError::InvalidStatusTransition(
                    format!("Can only cancel published events, current status: {}", self.status),
                ));
            }
        }

        self.status = EventStatus::Cancelled;
        self.updated_at = Utc::now();

        for category in &mut self.categories {
            category.is_active = false;
        }

        self.events.push(Box::new(EventCancelled {
            event_id: self.id,
        }));

        Ok(())
    }

    pub fn disable_category(&mut self, category_id: &CategoryId) -> Result<(), DomainError> {
        if self.status == EventStatus::Completed {
            return Err(DomainError::InvalidStatusTransition(
                "Cannot modify categories of a completed event".to_string(),
            ));
        }

        let category = self.categories.iter_mut()
            .find(|c| &c.id == category_id)
            .ok_or_else(|| DomainError::BusinessRule(
                format!("Category {} not found in event", category_id),
            ))?;

        category.is_active = false;
        self.updated_at = Utc::now();

        self.events.push(Box::new(TicketCategoryDisabled {
            event_id: self.id,
            category_id: *category_id,
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

    fn make_organizer() -> UserId {
        UserId::new()
    }

    fn make_event() -> Event {
        let now = Utc::now();
        Event::new(
            EventId::new(),
            make_organizer(),
            "Tech Conf".to_string(),
            "A cool conf".to_string(),
            "Jakarta".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            1000,
        )
        .unwrap()
    }

    fn add_category(event: &mut Event) -> CategoryId {
        let cat_id = CategoryId::new();
        let now = Utc::now();
        event
            .add_ticket_category(
                cat_id,
                "Regular".to_string(),
                Money::new(dec!(50.0), "IDR").unwrap(),
                100,
                now,
                now + Duration::days(5),
            )
            .unwrap();
        cat_id
    }

    #[test]
    fn create_event_fails_when_end_date_before_start_date() {
        let now = Utc::now();
        let result = Event::new(
            EventId::new(),
            make_organizer(),
            "Tech Conf".to_string(),
            "".to_string(),
            "Jakarta".to_string(),
            now + Duration::days(12),
            now + Duration::days(10),
            1000,
        );
        assert!(matches!(result, Err(DomainError::InvalidSchedule)));
    }

    #[test]
    fn create_event_fails_when_capacity_is_zero() {
        let now = Utc::now();
        let result = Event::new(
            EventId::new(),
            make_organizer(),
            "Tech Conf".to_string(),
            "".to_string(),
            "Jakarta".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            0,
        );
        assert!(matches!(result, Err(DomainError::InvalidCapacity)));
    }

    #[test]
    fn publish_event_fails_without_active_category() {
        let mut event = make_event();
        // No categories added
        let result = event.publish();
        assert!(matches!(result, Err(DomainError::BusinessRule(_))));
    }

    #[test]
    fn publish_event_fails_when_quota_exceeds_capacity() {
        let now = Utc::now();
        let mut event = Event::new(
            EventId::new(),
            make_organizer(),
            "Small Event".to_string(),
            "".to_string(),
            "Jakarta".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            50, // Small capacity
        )
        .unwrap();

        let result = event.add_ticket_category(
            CategoryId::new(),
            "VIP".to_string(),
            Money::new(dec!(100.0), "IDR").unwrap(),
            100,
            now,
            now + Duration::days(5),
        );
        assert!(matches!(result, Err(DomainError::QuotaExceedsCapacity)));
    }

    #[test]
    fn cancel_event_fails_when_completed() {
        let mut event = make_event();
        add_category(&mut event);
        event.publish().unwrap();
        event.status = EventStatus::Completed;

        let result = event.cancel();
        assert!(matches!(result, Err(DomainError::InvalidStatusTransition(_))));
    }

    #[test]
    fn cancelled_event_cannot_be_published() {
        let mut event = make_event();
        add_category(&mut event);
        event.publish().unwrap();
        event.cancel().unwrap();

        let result = event.publish();
        assert!(matches!(result, Err(DomainError::InvalidStatusTransition(_))));
    }

    #[test]
    fn create_and_publish_event_success() {
        let mut event = make_event();
        assert_eq!(event.status, EventStatus::Draft);
        assert_eq!(event.events.len(), 1); // EventCreated

        add_category(&mut event);
        assert_eq!(event.categories.len(), 1);

        event.publish().unwrap();
        assert_eq!(event.status, EventStatus::Published);
    }

    #[test]
    fn cancel_published_event_success() {
        let mut event = make_event();
        add_category(&mut event);
        event.publish().unwrap();
        event.cancel().unwrap();

        assert_eq!(event.status, EventStatus::Cancelled);
        assert!(event.categories.iter().all(|c| !c.is_active));
    }

    #[test]
    fn disable_category_success() {
        let mut event = make_event();
        let cat_id = add_category(&mut event);
        assert!(event.categories[0].is_active);

        event.disable_category(&cat_id).unwrap();
        assert!(!event.categories[0].is_active);
    }

    #[test]
    fn add_ticket_category_fails_exceeds_capacity() {
        let now = Utc::now();
        let mut event = Event::new(
            EventId::new(),
            make_organizer(),
            "Tech Conf".to_string(),
            "".to_string(),
            "Jakarta".to_string(),
            now + Duration::days(10),
            now + Duration::days(12),
            100,
        )
        .unwrap();

        let result = event.add_ticket_category(
            CategoryId::new(),
            "VIP".to_string(),
            Money::new(dec!(50.0), "IDR").unwrap(),
            150,
            now,
            now + Duration::days(5),
        );

        assert!(matches!(result, Err(DomainError::QuotaExceedsCapacity)));
    }
}
