use chrono::{DateTime, Utc};

use crate::shared::value_objects::Money;

use super::value_objects::{CategoryId, EventId};

#[derive(Debug, Clone)]
pub struct TicketCategory {
    pub id: CategoryId,
    pub event_id: EventId,
    pub name: String,
    pub price: Money,
    pub quota: u32,
    pub sold: u32,
    pub sales_start: DateTime<Utc>,
    pub sales_end: DateTime<Utc>,
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
            sold: 0,
            sales_start,
            sales_end,
        }
    }

    pub fn is_sold_out(&self) -> bool {
        self.sold >= self.quota
    }

    pub fn available_quota(&self) -> u32 {
        self.quota.saturating_sub(self.sold)
    }

    pub fn is_sales_active(&self, now: DateTime<Utc>) -> bool {
        now >= self.sales_start && now <= self.sales_end
    }

    pub fn add_sold(&mut self, quantity: u32) {
        self.sold += quantity;
    }
}
