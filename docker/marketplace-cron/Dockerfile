FROM --platform=$TARGETPLATFORM rust:latest as builder

WORKDIR /app

# Copy the entire project directory
COPY . .

# Build the application
RUN cargo build --release -p ark-marketplace-cron

# Start a new stage with a minimal image
FROM --platform=$TARGETPLATFORM debian:buster-slim

# Install SSL certificates and other necessary runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ark-marketplace-cron .

# Set the binary as the entrypoint
ENTRYPOINT ["./ark-marketplace-cron"]