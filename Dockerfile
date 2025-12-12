FROM rust:1.92-slim-trixie as builder

# could be "dev" for debug builds
ARG PROFILE=release

WORKDIR /oled
COPY ./Cargo.lock ./Cargo.toml ./

# pre-compile dependencies
RUN mkdir -p src && \
    echo 'fn main() {println!("wrong main!");}' > src/main.rs && \
    cargo build --profile=${PROFILE}

COPY ./src/ ./src/
RUN touch ./src/main.rs # tell cargo that the binary is outdated
RUN cargo build --profile=${PROFILE}
RUN mv target/*/oled ./

FROM debian:trixie as final

LABEL application=oled \
      version=unkown \
      maintainer=M3t0r

COPY --from=builder /oled/oled /usr/bin

ENTRYPOINT ["oled"]
CMD []
