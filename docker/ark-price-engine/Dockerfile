FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-price-engine --release

CMD ["cargo", "run", "-p", "ark-price-engine", "--release"]
