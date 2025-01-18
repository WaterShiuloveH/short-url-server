# Use Debian-based Rust image to ensure GLIBC compatibility
FROM rust:latest AS builder

# Create and set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files (to avoid rebuilding dependencies on every change)
COPY Cargo.toml Cargo.lock ./

# Copy the source code into the container
COPY src ./src

# Build the project
RUN cargo build --release

# Use the same Debian-based runtime image as Rust to ensure GLIBC compatibility
FROM debian:bookworm-slim

# Install required dependencies and SQLite3
RUN apt-get update && \
    apt-get install -y libssl-dev libc6 sqlite3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/url-shorter /usr/local/bin/url-shorter

# Copy the SQLite database file (if it already exists) to the container
# (You can adjust this path if your DB file is located elsewhere)
COPY ./urls.db /usr/src/app/urls.db

# Set the entry point to the application
CMD ["url-shorter"]
