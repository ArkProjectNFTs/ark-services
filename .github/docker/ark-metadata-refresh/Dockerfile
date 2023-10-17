FROM rust:latest

ARG AWS_ACCESS_KEY_ID
ENV AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID

ARG AWS_SECRET_ACCESS_KEY
ENV AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

WORKDIR /app

COPY . .

RUN cargo build -p ark-metadata-refresh --release

CMD ["cargo", "run", "-p", "ark-metadata-refresh", "--release"]