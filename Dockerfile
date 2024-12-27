# Use the official Rust image based on Alpine as the base image
FROM rust:alpine AS builder

# Install necessary dependencies for buildx
RUN apk add --no-cache \
    build-base \
    clang \
    cmake \
    openssl-dev \
    pkgconfig

# Create a new directory for the project
WORKDIR /usr/src/libplanet-remotekv

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs file to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies
RUN cargo build --release && rm -rf src

# Copy the source code
COPY . .

# Build the project
RUN cargo build --release

# Use a minimal Alpine image for the final stage
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache \
    libgcc \
    libstdc++

# OpenContainers annotations
LABEL org.opencontainers.image.title="libplanet-remotekv-rust"
LABEL org.opencontainers.image.description="A Rust project for libplanet-remotekv"
LABEL org.opencontainers.image.authors="Planetarium <engineering@planetariumhq.com>"
LABEL org.opencontainers.image.source="https://github.com/yourusername/libplanet-remotekv"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/libplanet-remotekv/target/release/libplanet-remotekv /usr/local/bin/libplanet-remotekv

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/libplanet-remotekv"]
