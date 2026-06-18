#!/bin/bash
set -e

# Load .env if it exists
if [ -f .env ]; then
    export $(cat .env | xargs)
fi

DATABASE_URL=${DATABASE_URL:-"postgres://mintcarbon:mintcarbon_dev@localhost:5432/mintcarbon"}

echo "Running migrations..."
sqlx migrate run --database-url "$DATABASE_URL"
echo "Migrations completed successfully."
