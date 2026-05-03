-- Refunds table: aggregate root for refund processing
CREATE TABLE refunds (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  booking_id        UUID NOT NULL REFERENCES bookings(id),
  customer_id       UUID NOT NULL,
  amount            NUMERIC(12,2) NOT NULL,
  currency          TEXT NOT NULL DEFAULT 'IDR',
  status            refund_status NOT NULL DEFAULT 'requested',
  rejection_reason  TEXT,
  payment_reference TEXT,
  requested_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
