FROM rust:latest

ARG AWS_ACCESS_KEY_ID
ENV AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID

ARG AWS_SECRET_ACCESS_KEY
ENV AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

WORKDIR /app

COPY . .

RUN cargo build -p arkchain-indexer --release

CMD ["cargo", "run", "-p", "arkchain-indexer", "--release"]

#aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 223605539824.dkr.ecr.us-east-1.amazonaws.com
