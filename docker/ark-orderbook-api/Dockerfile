FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-orderbook-api --release

CMD ["cargo", "run", "-p", "ark-orderbook-api", "--release"]
