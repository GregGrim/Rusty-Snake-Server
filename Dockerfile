FROM rust:latest

WORKDIR /rastach-server

COPY Cargo.toml Cargo.lock ./

# RUN cargo build --release

COPY . .

RUN cargo install --path .

ENTRYPOINT ["rastach-server"]