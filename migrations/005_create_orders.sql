CREATE TABLE orders (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id     UUID NOT NULL REFERENCES users(id),
    listing_id   UUID NOT NULL REFERENCES listings(id),
    quantity     BIGINT NOT NULL,
    total_lumens BIGINT NOT NULL,
    tx_hash      TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_orders_buyer ON orders (buyer_id);
CREATE INDEX idx_orders_listing ON orders (listing_id);
