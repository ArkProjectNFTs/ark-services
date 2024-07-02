FROM rust:latest

WORKDIR /app

COPY ark-marketplace-cron/ .

RUN cargo build -p ark-marketplace-cron --release

CMD ["cargo", "run", "-p", "ark-marketplace-cron", "--release"]
