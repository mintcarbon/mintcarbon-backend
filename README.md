# mintcarbon — backend

> Backend services for the mintcarbon platform — REST API, KYC/identity verification, Registry integrations, notification delivery, price oracle adapter, and compliance reporting infrastructure.

## Overview

This repository contains all off-chain services that support the mintcarbon platform. These services handle user registration, identity verification, external Registry API integration, notification delivery, market data aggregation, compliance report generation, and act as the bridge between the web frontend and the Soroban smart contracts.

## Services

### API Gateway

RESTful HTTP API consumed by the frontend and external integrators.

**Capabilities:**

- User registration and authentication (JWT + MFA)
- Protected endpoints enforcing RBAC (Issuer, Trader, Compliance_Officer, Administrator)
- TLS 1.3 enforcement on all connections
- Paginated, filterable data access for portfolios, listings, orders, and history
- CSV report download for transaction history
- Rate limiting and request validation
- Session token management with expiry-based re-authentication

### KYC Module

Identity verification subsystem integrated with third-party providers.

**Capabilities:**

- Collect email, full legal name, and jurisdiction during registration
- Submit identity data to third-party KYC provider (e.g., Onfido, Jumio)
- Accept or reject within 48-hour SLA
- Sanctioned-country check against maintained blocklist
- Encrypted document storage with 7-year retention minimum
- Temporary account restrictions while KYC is pending
- Webhook receiver for async KYC status updates

### Registry Adapter

Integration layer for external carbon credit registries.

**Capabilities:**

- Validate certificate IDs against Registry APIs (Verra VCS, Gold Standard, American Carbon Registry)
- Support minimum 3 registries at launch, extensible to additional registries
- Poll for certificate revocation status
- Parse Registry API responses into canonical `VerificationRecord` format
- Cache certificate validation results with TTL
- Raise events on certificate revocation for Suspension workflow

### Notification Service

Event-driven messaging subsystem.

**Capabilities:**

- Deliver email, in-app, and (optionally) SMS notifications
- Templates for: registration confirmation, KYC status change, order confirmation, listing created, certificate revocation, upgrade notification
- 60-second delivery SLA for order execution confirmations
- Retry with exponential backoff for failed deliveries
- Notification preference management per user
- Delivery status tracking

### Price Oracle Adapter

Bridge between external carbon pricing feeds and the platform.

**Capabilities:**

- Consume reference prices from external carbon price oracles (e.g., Toucan, CBL, ICE)
- Publish reference prices to the Marketplace (within 30-second SLA)
- Retain historical price data for 5+ years
- Provide queryable API for price history
- Support fallback pricing on oracle outage

### Audit Indexer

Off-chain index and query layer over on-chain AuditLog events.

**Capabilities:**

- Index all on-chain AuditLog events into a queryable database
- Generate compliance reports for specified date ranges within 60-second SLA
- Expose cryptographic Merkle-proofs for external verification
- Rebuild index from on-chain data if needed
- 10-year data retention

### Compliance Report Generator

Specialized report generation for regulatory use.

**Capabilities:**

- Per-user and platform-wide audit reports
- Double-counting detection and prevention reports
- Token supply reconciliation (minted vs retired vs circulating)
- Configurable date-range filtering
- Output formats: JSON, CSV, PDF

## Tech Stack

- **Language:** Rust (primary) / Go (secondary)
- **Database:** PostgreSQL 16
- **Cache / Queue:** Redis 7
- **Message Broker:** NATS or RabbitMQ
- **API Framework:** Axum (Rust) or Gin (Go)
- **Object Storage:** S3-compatible (MinIO for dev)
- **Containerization:** Docker + Docker Compose
- **Orchestration:** Kubernetes (production)

## Project Structure

```
mintcarbon-backend/
├── Cargo.toml                 # Workspace manifest
├── docker-compose.yml         # Local development environment
├── .env.example               # Template for environment variables
├── api/                       # RESTful HTTP API service (Axum)
│   └── src/
│       ├── db/                # Database models and access
│       ├── middleware/        # Auth, RBAC, etc.
│       └── routes/            # API endpoints
├── compliance/                # Compliance reporting module
├── indexer/                   # Audit log indexer
├── kyc/                       # KYC/Identity verification module
├── notification/              # Notification delivery service
├── oracle/                    # Price oracle adapter
├── registry/                  # External registry integration
├── migrations/                # PostgreSQL migrations (SQLx)
├── scripts/                   # Helper scripts (setup, migrations)
└── LICENSE                    # MIT License
```

## Prerequisites

- Rust 1.77+
- PostgreSQL 16
- Redis 7
- Docker + Docker Compose
- `sqlx-cli` (`cargo install sqlx-cli`)

## Getting Started

1. **Clone the repository and set up environment variables:**

```bash
cp .env.example .env
# Edit .env with your local settings
```

2. **Start infrastructure (Postgres, Redis, MinIO):**

```bash
docker compose up -d
```

3. **Run database migrations:**

```bash
./scripts/migrate.sh
```

4. **Seed test data (optional):**

```bash
./scripts/seed.sh
```

5. **Start the services:**

```bash
# Start the API server
cargo run -p mintcarbon-api

# Start the indexer (in a separate terminal)
cargo run -p mintcarbon-indexer
```

## Configuration

Configuration is managed via TOML files and environment variable overrides.

| Variable                  | Description                      | Default                                                      |
| ------------------------- | -------------------------------- | ------------------------------------------------------------ |
| `DATABASE_URL`            | PostgreSQL connection string     | `postgres://mintcarbon:mintcarbon@localhost:5432/mintcarbon` |
| `REDIS_URL`               | Redis connection string          | `redis://localhost:6379`                                     |
| `SOROBAN_RPC_URL`         | Soroban RPC endpoint             | `https://rpc.testnet.stellar.org`                            |
| `KYC_PROVIDER_API_KEY`    | Third-party KYC API key          | —                                                            |
| `NOTIFICATION_EMAIL_FROM` | Sender email address             | `noreply@mintcarbon.io`                                      |
| `VERRA_API_KEY`           | Verra Registry API key           | —                                                            |
| `GOLD_STANDARD_API_KEY`   | Gold Standard API key            | —                                                            |
| `ACR_API_KEY`             | American Carbon Registry API key | —                                                            |
| `JWT_SECRET`              | JWT signing secret               | — (required)                                                 |
| `ENCRYPTION_KEY`          | KYC document encryption key      | — (required)                                                 |
| `RUST_LOG`                | Log level                        | `info`                                                       |

## API Endpoints

### Authentication & Users

| Method | Path                      | Auth | Description              |
| ------ | ------------------------- | ---- | ------------------------ |
| POST   | `/api/v1/auth/register`   | None | Register new user        |
| POST   | `/api/v1/auth/login`      | None | Login (returns JWT)      |
| POST   | `/api/v1/auth/mfa/setup`  | User | Enable MFA               |
| POST   | `/api/v1/auth/mfa/verify` | User | Verify MFA token         |
| GET    | `/api/v1/users/me`        | User | Get current user profile |

### Projects

| Method | Path                   | Auth   | Description          |
| ------ | ---------------------- | ------ | -------------------- |
| POST   | `/api/v1/projects`     | Issuer | Register new project |
| GET    | `/api/v1/projects`     | Any    | List projects        |
| GET    | `/api/v1/projects/:id` | Any    | Get project details  |

### Tokens

| Method | Path                    | Auth   | Description                          |
| ------ | ----------------------- | ------ | ------------------------------------ |
| POST   | `/api/v1/tokens/mint`   | Issuer | Mint tokens against verified project |
| POST   | `/api/v1/tokens/retire` | User   | Retire tokens                        |
| GET    | `/api/v1/tokens`        | User   | List user's token holdings           |
| GET    | `/api/v1/tokens/:id`    | User   | Get token details                    |

### Marketplace

| Method | Path                   | Auth   | Description                          |
| ------ | ---------------------- | ------ | ------------------------------------ |
| GET    | `/api/v1/listings`     | Any    | List active listings                 |
| POST   | `/api/v1/listings`     | User   | Create listing                       |
| DELETE | `/api/v1/listings/:id` | Seller | Cancel listing                       |
| POST   | `/api/v1/orders`       | User   | Place order                          |
| GET    | `/api/v1/market/data`  | Any    | Market data (best ask, volume, etc.) |

### Compliance

| Method | Path                                  | Auth               | Description          |
| ------ | ------------------------------------- | ------------------ | -------------------- |
| GET    | `/api/v1/compliance/reports`          | Compliance_Officer | Request audit report |
| GET    | `/api/v1/compliance/audit-log`        | Compliance_Officer | Query audit log      |
| GET    | `/api/v1/compliance/proofs/:entry_id` | Compliance_Officer | Get Merkle proof     |

### Portfolio

| Method | Path                        | Auth | Description                       |
| ------ | --------------------------- | ---- | --------------------------------- |
| GET    | `/api/v1/portfolio`         | User | Token balances grouped by project |
| GET    | `/api/v1/portfolio/history` | User | Paginated transaction history     |
| GET    | `/api/v1/portfolio/export`  | User | Download CSV report               |

## Security

- **TLS 1.3** — All endpoints enforce TLS
- **JWT + MFA** — Session tokens with multi-factor authentication for sensitive roles
- **RBAC** — Role-based access control at middleware level
- **Rate limiting** — Per-endpoint, per-IP, per-user rate limits
- **Input validation** — All request bodies validated against schemas
- **Encryption at rest** — KYC documents encrypted with AES-256-GCM
- **Malware scanning** — All uploaded documents scanned before storage
- **Audit logging** — All state-changing requests logged to AuditLog

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests (requires Docker)
cargo test --test '*'

# Run e2e tests (requires full stack)
cargo test --test 'e2e/*'
```

## Registry Certificate JSON Schema

The Registry certificate parser supports the following JSON schema (Req 12):

```json
{
    "registry": "Verra VCS | Gold Standard | American Carbon Registry",
    "certificate_id": "VCS-1234",
    "project_name": "Amazon Rainforest Conservation",
    "project_type": "REDD+ | Renewable Energy | Methane Capture | ...",
    "location": {
        "country": "Brazil",
        "region": "Para"
    },
    "vintage_year": 2024,
    "issuance_date": "2025-01-15",
    "credits": 10000,
    "status": "active | revoked | expired"
}
```

The `RegistryParser` converts this into a `VerificationRecord`, and the `PrettyPrinter` serializes it back. The round-trip property holds: `parse(print(parse(json))) == parse(json)`.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

This project is licensed under the [MIT License](LICENSE).
