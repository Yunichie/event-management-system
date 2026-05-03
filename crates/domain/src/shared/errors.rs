/// Errors originating from domain business rule violations.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("End date must be after start date")]
    InvalidSchedule,
    #[error("Capacity must be greater than zero")]
    InvalidCapacity,
    #[error("Ticket price cannot be negative")]
    NegativePrice,
    #[error("Ticket quota must be greater than zero")]
    InvalidQuota,
    #[error("Total category quota exceeds event capacity")]
    QuotaExceedsCapacity,
    #[error("Sales end date must be on or before event start date")]
    InvalidSalesPeriod,
    #[error("Invalid status transition: {0}")]
    InvalidStatusTransition(String),
    #[error("Payment deadline has passed")]
    PaymentDeadlinePassed,
    #[error("Payment amount does not match booking total")]
    PaymentAmountMismatch,
    #[error("Ticket has already been checked in")]
    AlreadyCheckedIn,
    #[error("Ticket does not match this event")]
    TicketEventMismatch,
    #[error("Refund cannot be requested: a ticket has already been checked in")]
    CheckedInTicketRefundDenied,
    #[error("Rejection reason is required")]
    RejectionReasonRequired,
    #[error("Refund is not in Requested status")]
    RefundNotRequested,
    #[error("Refund is not in Approved status")]
    RefundNotApproved,
    #[error("Money amounts have different currencies")]
    CurrencyMismatch,
    #[error("Money amount cannot be negative")]
    NegativeMoney,
    #[error("{0}")]
    BusinessRule(String),
}

/// Errors originating from repository operations (persistence layer).
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Entity not found")]
    NotFound,
}
