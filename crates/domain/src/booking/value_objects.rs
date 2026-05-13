use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookingId(Uuid);

impl BookingId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for BookingId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for BookingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingStatus {
    PendingPayment,
    Paid,
    Expired,
    Refunded,
}

impl std::fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingStatus::PendingPayment => write!(f, "PendingPayment"),
            BookingStatus::Paid => write!(f, "Paid"),
            BookingStatus::Expired => write!(f, "Expired"),
            BookingStatus::Refunded => write!(f, "Refunded"),
        }
    }
}
