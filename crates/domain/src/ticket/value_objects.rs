use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketId(Uuid);

impl TicketId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TicketId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketCode(String);

impl TicketCode {
    pub fn new(code: String) -> Self {
        Self(code)
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string().to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TicketCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Active,
    CheckedIn,
    Cancelled,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::Active => write!(f, "Active"),
            TicketStatus::CheckedIn => write!(f, "CheckedIn"),
            TicketStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}
