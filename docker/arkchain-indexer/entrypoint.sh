#!/bin/bash

echo "Execute migrations..."
cd migrations
sqlx migrate run --source orderbook

exec "$@"
