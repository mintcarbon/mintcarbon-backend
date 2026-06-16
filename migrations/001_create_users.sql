CREATE TYPE user_role AS ENUM ('issuer', 'trader', 'compliance_officer', 'administrator');
CREATE TYPE kyc_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role        user_role NOT NULL DEFAULT 'trader',
    mfa_secret  TEXT,
    kyc_status  kyc_status NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_kyc_status ON users (kyc_status);
