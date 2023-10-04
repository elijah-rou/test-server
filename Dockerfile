# BUILD BASE
FROM rust:bookworm as build

# create a new empty shell project
RUN USER=root cargo new --bin test_server
WORKDIR /test_server
RUN update-ca-certificates

# copy over your manifests
COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY src ./src
RUN touch ./src/main.rs

# build for release
RUN rm ./target/release/deps/test_server*
RUN cargo build --release


# FINAL BASE
FROM debian:bookworm-slim as final
RUN apt-get update && apt-get -y install openssl
WORKDIR /app

# copy the build artifact from the build stage and ca-certificates for SSL
COPY --from=build  /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /test_server/target/release/test_server .

# set the startup command to run your binary
CMD ["./test_server"]
