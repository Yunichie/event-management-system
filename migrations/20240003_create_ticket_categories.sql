-- Ticket categories: entity within the Event aggregate
CREATE TABLE ticket_categories (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id         UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  name             TEXT NOT NULL,
  price_amount     NUMERIC(12,2) NOT NULL CHECK (price_amount >= 0),
  price_currency   TEXT NOT NULL DEFAULT 'IDR',
  quota            INT NOT NULL CHECK (quota > 0),
  remaining_quota  INT NOT NULL,
  sales_start      DATE NOT NULL,
  sales_end        DATE NOT NULL,
  is_active        BOOLEAN NOT NULL DEFAULT TRUE,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
