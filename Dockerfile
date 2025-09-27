# Development stage
FROM rust:1.71 as backend-builder
WORKDIR /usr/src/backend
COPY Backend/Cargo.toml Backend/Cargo.lock ./
COPY Backend/src ./src
RUN cargo build --release

# Frontend builder stage
FROM rust:1.71 as frontend-builder
WORKDIR /usr/src/frontend
RUN cargo install trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

COPY chat/Cargo.toml chat/Cargo.lock ./
COPY chat/src ./src
COPY chat/index.html chat/styles.css ./
RUN trunk build --release

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app

# Copy the built executables from builder stages
COPY --from=backend-builder /usr/src/backend/target/release/Backend ./backend
COPY --from=frontend-builder /usr/src/frontend/dist ./frontend/dist

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Expose ports
EXPOSE 8080

# Start both services
CMD ["./backend"]