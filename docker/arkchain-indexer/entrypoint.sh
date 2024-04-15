#!/bin/bash

echo "Execute migrations..."
sqlx migrate run --source orderbook

exec "$@"
