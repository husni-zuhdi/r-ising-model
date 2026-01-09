# Stage 1: Build trunk distribution
FROM rust:1.89.0-bookworm AS builder
COPY . .
# Setup build dependencies
RUN rustup target add wasm32-unknown-unknown
RUN wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- && mv trunk /usr/bin
# Stage 1.1: Build distribution
RUN cd gui && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk build --release --minify --locked --dist ../web/dist
# Stage 1.2: Build webapp
RUN cargo build --package web --locked --release

# Stage 2.1: Run web binary
FROM gcr.io/distroless/cc
COPY --from=builder ./web/dist /dist
COPY --from=builder ./target/release/web /web
CMD ["/web"]
