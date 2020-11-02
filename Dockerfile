# https://gitlab.com/rust_musl_docker/image
FROM registry.gitlab.com/rust_musl_docker/image:stable-latest AS builder

# Compiles our deps first, this will save a lot in build time
WORKDIR /usr/src/
RUN rustup target add x86_64-unknown-linux-musl
RUN USER=root cargo new cfddns
WORKDIR /usr/src/cfddns
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release -vv --target=x86_64-unknown-linux-musl

# Compile cfddns
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Copy static binary into scratch
FROM scratch
COPY --from=builder /usr/src/cfddns/target/x86_64-unknown-linux-musl/release/cfddns /
USER 1000
CMD ["/cfddns"]
