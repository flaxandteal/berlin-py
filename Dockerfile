FROM rust:1.58 as chef
WORKDIR /berlin-rs
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# Build dependencies - this is the caching Docker layer
FROM chef AS deps-builder
COPY --from=planner /berlin-rs/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Actually build with our source code (not only deps)
FROM deps-builder as builder
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary
FROM debian:bullseye-slim
WORKDIR /berlin-rs
COPY --from=builder /berlin-rs/target/release/berlin-web .
COPY --from=builder /berlin-rs/data/ ./data/
ENTRYPOINT ["./berlin-web"]
