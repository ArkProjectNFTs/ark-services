#!/bin/bash

echo "Execute migrations..."
sqlx migrate run

exec "$@"
