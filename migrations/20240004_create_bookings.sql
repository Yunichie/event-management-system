-- Bookings table: aggregate root for ticket reservations and payments
CREATE TABLE bookings (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  customer_id      UUID NOT NULL,
  event_id         UUID NOT NULL REFERENCES events(id),
  category_id      UUID NOT NULL REFERENCES ticket_categories(id),
  quantity         INT NOT NULL CHECK (quantity > 0),
  total_amount     NUMERIC(12,2) NOT NULL,
  total_currency   TEXT NOT NULL DEFAULT 'IDR',
  status           booking_status NOT NULL DEFAULT 'pending_payment',
  payment_deadline TIMESTAMPTZ NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (customer_id, event_id, status)  -- only one active booking per customer per event
);
