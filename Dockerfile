FROM clux/muslrust:nightly AS planner
WORKDIR /app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN rustup show
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies - this is the caching Docker layer!
FROM clux/muslrust:nightly AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build the application
FROM clux/muslrust:nightly AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /root/.cargo /root/.cargo
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --bin codestrain-server --release --target x86_64-unknown-linux-musl

# We don't need the Rust toolchain to run the binary!
FROM gcr.io/distroless/static:nonroot
ARG DATABASE_URL
WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/codestrain-server /app/codestrain-server
ENV ROCKET_port="8080"
CMD ["/app/codestrain-server"]