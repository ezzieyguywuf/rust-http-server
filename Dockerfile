# Use the official Rust image.
# https://hub.docker.com/_/rust
FROM ubuntu:latest

# Copy local code to the container image.
WORKDIR /usr/src/app
COPY . .

# Install production dependencies and build a release artifact.
RUN apt-get update && apt-get  install -y cargo && cargo install --path .

# Service must listen to $PORT environment variable.
# This default value facilitates local development.
ENV PORT 7878

# Run the web service on container startup.
CMD /root/.cargo/bin/rust-http-server --port ${PORT} --address "0.0.0.0" --name '$(hostname)'
