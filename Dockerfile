FROM rust:1.67

COPY config config
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Build your program for release
RUN cargo build --release

# Run the binary
ENTRYPOINT ["./target/release/discord-playground"]