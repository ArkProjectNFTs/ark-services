FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-marketplace-cron --release

CMD ["cargo", "run", "-p", "ark-marketplace-cron", "--release"]
