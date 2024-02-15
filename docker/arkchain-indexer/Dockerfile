FROM rust:latest

ARG AWS_ACCESS_KEY_ID
ENV AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID

ARG AWS_SECRET_ACCESS_KEY
ENV AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

WORKDIR /app

COPY . .

# copy migration files
COPY ../ark-sqlx/migrations ./migrations
RUN cargo install sqlx-cli --locked

COPY docker/arkchain-indexer/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]


RUN cargo build -p arkchain-indexer --release

CMD ["cargo", "run", "-p", "arkchain-indexer", "--release"]

#aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 223605539824.dkr.ecr.us-east-1.amazonaws.com
