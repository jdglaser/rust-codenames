# Build Frontend
FROM node:16 as frontend

WORKDIR /usr/src/app/frontend
COPY ./frontend .

RUN npm install
RUN npm run build-docker

# Build Rust app
FROM rust:1.62 as app

WORKDIR /usr/src/app
COPY --from=frontend /usr/src/app/frontend/dist/ /usr/src/app/dist/
COPY ./app /usr/src/app/

RUN cargo build -r

# Run app
FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=app /usr/src/app /usr/src/app
WORKDIR /usr/src/app
CMD ["./target/release/rust-codenames"]
