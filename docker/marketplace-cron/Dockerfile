FROM --platform=$TARGETPLATFORM rust:latest as builder

WORKDIR /app

# Copy only the necessary files for the cron job
COPY ark-marketplace-cron/Cargo.toml ./Cargo.toml
COPY ark-marketplace-cron/src ./src
COPY .sqlx ./.sqlx

# Create a dummy Cargo.lock to satisfy the compiler
RUN touch Cargo.lock

# Build dependencies
RUN cargo build --release

# Build the actual application
RUN cargo build --release

# Start a new stage with a minimal image
FROM --platform=$TARGETPLATFORM debian:buster-slim

# Install SSL certificates and other necessary runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/ark-marketplace-cron .

# Set the binary as the entrypoint
CMD ["./ark-marketplace-cron"]