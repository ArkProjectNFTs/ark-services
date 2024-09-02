FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build -p ark-priceengine --release

CMD ["cargo", "run", "-p", "ark-priceengine", "--release"]
