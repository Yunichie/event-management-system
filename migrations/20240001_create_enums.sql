CREATE TYPE event_status   AS ENUM ('draft', 'published', 'cancelled', 'completed');
CREATE TYPE booking_status AS ENUM ('pending_payment', 'paid', 'expired', 'refunded');
CREATE TYPE ticket_status  AS ENUM ('active', 'checked_in', 'cancelled');
CREATE TYPE refund_status  AS ENUM ('requested', 'approved', 'rejected', 'paid_out');
