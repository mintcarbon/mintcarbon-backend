CREATE TYPE listing_status AS ENUM ('active', 'filled', 'cancelled');

CREATE TABLE listings (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id          UUID NOT NULL REFERENCES users(id),
    token_id           TEXT NOT NULL,
    quantity           BIGINT NOT NULL,
    price_lumens       BIGINT NOT NULL,
    status             listing_status NOT NULL DEFAULT 'active',
    contract_listing_id TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_listings_seller ON listings (seller_id);
CREATE INDEX idx_listings_status ON listings (status);
CREATE INDEX idx_listings_token ON listings (token_id);
