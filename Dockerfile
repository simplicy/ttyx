# Build stage
FROM rust:1.90.0-slim-bullseye AS build

# Install necessary dependencies for build stage
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

# Clone repository into image and build
COPY Cargo.lock Cargo.toml ./
COPY index.html index.html ./
# Build the actual project
COPY src src
RUN trunk build --locked --release
RUN cp -r ./dist /app

# Final image
FROM debian:bullseye-slim AS final

# Copy the compiled binary
COPY --from=build /app /app
RUN apt-get update 
RUN apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
RUN curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
RUN curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
RUN chmod o+r /usr/share/keyrings/caddy-stable-archive-keyring.gpg
RUN chmod o+r /etc/apt/sources.list.d/caddy-stable.list
RUN apt update
RUN apt install caddy
RUN cd /app
# Start caddy static file server without tls
CMD caddy file-server --browse --root /app --listen :80
