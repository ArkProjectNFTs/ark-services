FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-marketplace-cron --release


COPY --from=builder /app/target/release/ark-marketplace-cron .