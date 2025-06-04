FROM rust AS build

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release
RUN cargo install --root /.cargo/ diesel_cli --no-default-features --features postgres

FROM debian:stable-slim
COPY --from=build /target/release/recipes-rs .
COPY --from=build /.cargo/bin/diesel ./bin/

COPY ./migrations ./migrations
COPY ./diesel.toml .
COPY ./bin/entrypoint.sh .

# diesel dependency
RUN apt update -y && apt install libpq5 -y

EXPOSE 8080

ENTRYPOINT ["./entrypoint.sh"]
