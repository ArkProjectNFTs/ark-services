FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-indexer --release

CMD ["cargo", "run", "-p", "ark-indexer", "--release"]