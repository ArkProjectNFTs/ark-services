FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-marketplace-api --release

CMD ["cargo", "run", "-p", "ark-marketplace-api", "--release"]
