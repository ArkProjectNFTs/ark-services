#!/bin/bash

echo "DATABASE_URL: $DATABASE_URL"

echo "Execute migrations..."
sqlx migrate run

exec "$@"
