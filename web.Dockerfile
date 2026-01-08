# Stage 1: Build trunk distribution
FROM rust:1.89.0-bookworm AS builder
COPY . .
# Setup build dependencies
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall trunk

# Stage 1.1: Build distribution
RUN cd gui && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk build --release --minify --locked --dist ../web/dist
# Stage 1.2: Build webapp
RUN cargo build --package web --locked --release

# Stage 2.1: Run web binary
FROM gcr.io/distroless/cc
COPY --from=builder ./web/dist /dist
COPY --from=builder ./target/release/web /web
CMD ["/web"]
