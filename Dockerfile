FROM clux/muslrust:latest as builder
LABEL authors="Mr. Liid"
WORKDIR /app

COPY Cargo.toml rust-toolchain.toml ./
COPY crates/parser/Cargo.toml crates/parser/

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir -p crates/parser/src && touch crates/parser/src/lib.rs
RUN cargo fetch --target x86_64-unknown-linux-musl
RUN rm -rf src
RUN rm -rf crates/parser/src

COPY src src
COPY crates/parser/src crates/parser/src


RUN cargo build --target x86_64-unknown-linux-musl --release --offline

FROM scratch
COPY ui ui
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/metagraph-db .

CMD ["./metagraph-db"]