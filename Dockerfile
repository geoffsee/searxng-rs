# Build stage
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

ENV OPENSSL_STATIC=1

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage - minimal image
FROM scratch

# Copy CA certificates for HTTPS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy binary
COPY --from=builder /app/target/release/searxng-rs /searxng-rs

EXPOSE 8080

ENTRYPOINT ["/searxng-rs"]
