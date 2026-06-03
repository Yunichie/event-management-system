use chrono::{DateTime, NaiveTime, TimeZone, Utc};

use domain::{
    booking::{
        aggregate::Booking,
        value_objects::{BookingId, BookingStatus},
    },
    event::{
        aggregate::Event,
        ticket_category::TicketCategory,
        value_objects::{CategoryId, EventId, EventStatus},
    },
    refund::{
        aggregate::Refund,
        value_objects::{RefundId, RefundStatus},
    },
    shared::value_objects::{Money, UserId},
    ticket::{
        entity::Ticket,
        value_objects::{TicketCode, TicketId, TicketStatus},
    },
};

use super::models::{
    BookingRow, BookingStatusDb, EventRow, EventStatusDb, RefundRow, RefundStatusDb,
    TicketCategoryRow, TicketRow, TicketStatusDb, TicketWithCategoryRow,
};

// Date helpers

fn naive_date_to_utc(date: chrono::NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN))
}

fn utc_to_naive_date(dt: DateTime<Utc>) -> chrono::NaiveDate {
    dt.date_naive()
}

impl From<EventStatus> for EventStatusDb {
    fn from(status: EventStatus) -> Self {
        match status {
            EventStatus::Draft => EventStatusDb::Draft,
            EventStatus::Published => EventStatusDb::Published,
            EventStatus::Cancelled => EventStatusDb::Cancelled,
            EventStatus::Completed => EventStatusDb::Completed,
        }
    }
}

impl From<EventStatusDb> for EventStatus {
    fn from(status: EventStatusDb) -> Self {
        match status {
            EventStatusDb::Draft => EventStatus::Draft,
            EventStatusDb::Published => EventStatus::Published,
            EventStatusDb::Cancelled => EventStatus::Cancelled,
            EventStatusDb::Completed => EventStatus::Completed,
        }
    }
}

impl From<BookingStatus> for BookingStatusDb {
    fn from(status: BookingStatus) -> Self {
        match status {
            BookingStatus::PendingPayment => BookingStatusDb::PendingPayment,
            BookingStatus::Paid => BookingStatusDb::Paid,
            BookingStatus::Expired => BookingStatusDb::Expired,
            BookingStatus::Refunded => BookingStatusDb::Refunded,
        }
    }
}

impl From<BookingStatusDb> for BookingStatus {
    fn from(status: BookingStatusDb) -> Self {
        match status {
            BookingStatusDb::PendingPayment => BookingStatus::PendingPayment,
            BookingStatusDb::Paid => BookingStatus::Paid,
            BookingStatusDb::Expired => BookingStatus::Expired,
            BookingStatusDb::Refunded => BookingStatus::Refunded,
        }
    }
}

impl From<TicketStatus> for TicketStatusDb {
    fn from(status: TicketStatus) -> Self {
        match status {
            TicketStatus::Active => TicketStatusDb::Active,
            TicketStatus::CheckedIn => TicketStatusDb::CheckedIn,
            TicketStatus::Cancelled => TicketStatusDb::Cancelled,
        }
    }
}

impl From<TicketStatusDb> for TicketStatus {
    fn from(status: TicketStatusDb) -> Self {
        match status {
            TicketStatusDb::Active => TicketStatus::Active,
            TicketStatusDb::CheckedIn => TicketStatus::CheckedIn,
            TicketStatusDb::Cancelled => TicketStatus::Cancelled,
        }
    }
}

impl From<RefundStatus> for RefundStatusDb {
    fn from(status: RefundStatus) -> Self {
        match status {
            RefundStatus::Requested => RefundStatusDb::Requested,
            RefundStatus::Approved => RefundStatusDb::Approved,
            RefundStatus::Rejected => RefundStatusDb::Rejected,
            RefundStatus::PaidOut => RefundStatusDb::PaidOut,
        }
    }
}

impl From<RefundStatusDb> for RefundStatus {
    fn from(status: RefundStatusDb) -> Self {
        match status {
            RefundStatusDb::Requested => RefundStatus::Requested,
            RefundStatusDb::Approved => RefundStatus::Approved,
            RefundStatusDb::Rejected => RefundStatus::Rejected,
            RefundStatusDb::PaidOut => RefundStatus::PaidOut,
        }
    }
}

// Event mapping

pub fn event_from_rows(row: EventRow, category_rows: Vec<TicketCategoryRow>) -> Event {
    let categories = category_rows
        .into_iter()
        .map(ticket_category_from_row)
        .collect();

    Event {
        id: EventId::from(row.id),
        organizer_id: UserId::from(row.organizer_id),
        name: row.name,
        description: row.description,
        location: row.location,
        start_date: naive_date_to_utc(row.start_date),
        end_date: naive_date_to_utc(row.end_date),
        max_capacity: row.max_capacity as u32,
        status: EventStatus::from(row.status),
        categories,
        created_at: row.created_at,
        updated_at: row.updated_at,
        events: Vec::new(),
    }
}

pub fn event_to_row(event: &Event) -> EventRow {
    EventRow {
        id: event.id.into_inner(),
        organizer_id: event.organizer_id.into_inner(),
        name: event.name.clone(),
        description: event.description.clone(),
        start_date: utc_to_naive_date(event.start_date),
        end_date: utc_to_naive_date(event.end_date),
        location: event.location.clone(),
        max_capacity: event.max_capacity as i32,
        status: EventStatusDb::from(event.status),
        created_at: event.created_at,
        updated_at: event.updated_at,
    }
}

// TicketCategory mapping

pub fn ticket_category_from_row(row: TicketCategoryRow) -> TicketCategory {
    TicketCategory {
        id: CategoryId::from(row.id),
        event_id: EventId::from(row.event_id),
        name: row.name,
        price: Money::new(row.price_amount, row.price_currency)
            .expect("DB price_amount should be non-negative"),
        quota: row.quota as u32,
        remaining_quota: row.remaining_quota as u32,
        sales_start: naive_date_to_utc(row.sales_start),
        sales_end: naive_date_to_utc(row.sales_end),
        is_active: row.is_active,
    }
}

pub fn ticket_category_to_row(cat: &TicketCategory) -> TicketCategoryRow {
    TicketCategoryRow {
        id: cat.id.into_inner(),
        event_id: cat.event_id.into_inner(),
        name: cat.name.clone(),
        price_amount: cat.price.amount(),
        price_currency: cat.price.currency().to_string(),
        quota: cat.quota as i32,
        remaining_quota: cat.remaining_quota as i32,
        sales_start: utc_to_naive_date(cat.sales_start),
        sales_end: utc_to_naive_date(cat.sales_end),
        is_active: cat.is_active,
        created_at: Utc::now(),
    }
}

// Booking mapping

pub fn booking_from_row(row: BookingRow, tickets: Vec<Ticket>) -> Booking {
    Booking {
        id: BookingId::from(row.id),
        user_id: UserId::from(row.customer_id),
        event_id: EventId::from(row.event_id),
        category_id: CategoryId::from(row.category_id),
        quantity: row.quantity as u32,
        total_amount: Money::new(row.total_amount, row.total_currency)
            .expect("DB total_amount should be non-negative"),
        status: BookingStatus::from(row.status),
        payment_deadline: row.payment_deadline,
        tickets,
        created_at: row.created_at,
        updated_at: row.updated_at,
        events: Vec::new(),
    }
}

pub fn booking_to_row(booking: &Booking) -> BookingRow {
    BookingRow {
        id: booking.id.into_inner(),
        customer_id: booking.user_id.into_inner(),
        event_id: booking.event_id.into_inner(),
        category_id: booking.category_id.into_inner(),
        quantity: booking.quantity as i32,
        total_amount: booking.total_amount.amount(),
        total_currency: booking.total_amount.currency().to_string(),
        status: BookingStatusDb::from(booking.status),
        payment_deadline: booking.payment_deadline,
        created_at: booking.created_at,
        updated_at: booking.updated_at,
    }
}

// Ticket mapping

pub fn ticket_from_row_with_category(row: TicketWithCategoryRow) -> Ticket {
    Ticket {
        id: TicketId::from(row.id),
        booking_id: BookingId::from(row.booking_id),
        event_id: EventId::from(row.event_id),
        category_id: CategoryId::from(row.category_id),
        code: TicketCode::new(row.code),
        status: TicketStatus::from(row.status),
        checked_in_at: row.checked_in_at,
        created_at: row.created_at,
        events: Vec::new(),
    }
}

pub fn ticket_from_row(row: TicketRow, category_id: CategoryId) -> Ticket {
    Ticket {
        id: TicketId::from(row.id),
        booking_id: BookingId::from(row.booking_id),
        event_id: EventId::from(row.event_id),
        category_id,
        code: TicketCode::new(row.code),
        status: TicketStatus::from(row.status),
        checked_in_at: row.checked_in_at,
        created_at: row.created_at,
        events: Vec::new(),
    }
}

pub fn ticket_to_row(ticket: &Ticket) -> TicketRow {
    TicketRow {
        id: ticket.id.into_inner(),
        booking_id: ticket.booking_id.into_inner(),
        event_id: ticket.event_id.into_inner(),
        code: ticket.code.as_str().to_string(),
        status: TicketStatusDb::from(ticket.status),
        checked_in_at: ticket.checked_in_at,
        created_at: ticket.created_at,
    }
}

// Refund mapping

pub fn refund_from_row(row: RefundRow) -> Refund {
    Refund {
        id: RefundId::from(row.id),
        booking_id: BookingId::from(row.booking_id),
        user_id: UserId::from(row.customer_id),
        amount: Money::new(row.amount, row.currency)
            .expect("DB refund amount should be non-negative"),
        status: RefundStatus::from(row.status),
        reason: None,
        rejection_reason: row.rejection_reason,
        payment_reference: row.payment_reference,
        requested_at: row.requested_at,
        events: Vec::new(),
    }
}

pub fn refund_to_row(refund: &Refund) -> RefundRow {
    RefundRow {
        id: refund.id.into_inner(),
        booking_id: refund.booking_id.into_inner(),
        customer_id: refund.user_id.into_inner(),
        amount: refund.amount.amount(),
        currency: refund.amount.currency().to_string(),
        status: RefundStatusDb::from(refund.status),
        rejection_reason: refund.rejection_reason.clone(),
        payment_reference: refund.payment_reference.clone(),
        requested_at: refund.requested_at,
        updated_at: Utc::now(),
    }
}
