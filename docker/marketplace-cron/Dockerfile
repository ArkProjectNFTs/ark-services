FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-marketplace-cron --release

# Remove any CMD or ENTRYPOINT