use chrono::{DateTime, Utc};

use crate::shared::{errors::DomainError, value_objects::Money};

use super::value_objects::{CategoryId, EventId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryDisplayStatus {
    Available,
    ComingSoon,
    SalesClosed,
    SoldOut,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct TicketCategory {
    pub id: CategoryId,
    pub event_id: EventId,
    pub name: String,
    pub price: Money,
    pub quota: u32,
    pub remaining_quota: u32,
    pub sales_start: DateTime<Utc>,
    pub sales_end: DateTime<Utc>,
    pub is_active: bool,
}

impl TicketCategory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CategoryId,
        event_id: EventId,
        name: String,
        price: Money,
        quota: u32,
        sales_start: DateTime<Utc>,
        sales_end: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            event_id,
            name,
            price,
            quota,
            remaining_quota: quota,
            sales_start,
            sales_end,
            is_active: true,
        }
    }

    pub fn is_available_now(&self, now: DateTime<Utc>) -> bool {
        self.is_active
            && now >= self.sales_start
            && now <= self.sales_end
            && self.remaining_quota > 0
    }

    pub fn reserve(&mut self, qty: u32) -> Result<(), DomainError> {
        if qty > self.remaining_quota {
            return Err(DomainError::BusinessRule(format!(
                "Requested quantity {} exceeds remaining quota {}",
                qty, self.remaining_quota
            )));
        }
        self.remaining_quota -= qty;
        Ok(())
    }

    pub fn release(&mut self, qty: u32) {
        self.remaining_quota = (self.remaining_quota + qty).min(self.quota);
    }

    pub fn display_status(&self, now: DateTime<Utc>) -> CategoryDisplayStatus {
        if !self.is_active {
            return CategoryDisplayStatus::Inactive;
        }
        if now < self.sales_start {
            return CategoryDisplayStatus::ComingSoon;
        }
        if now > self.sales_end {
            return CategoryDisplayStatus::SalesClosed;
        }
        if self.remaining_quota == 0 {
            return CategoryDisplayStatus::SoldOut;
        }
        CategoryDisplayStatus::Available
    }
}
