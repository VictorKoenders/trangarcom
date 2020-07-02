FROM alpine:latest AS builder

WORKDIR /app

RUN apk update
RUN apk upgrade
RUN apk add curl libgcc gcc libc-dev perl make
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable --profile minimal

COPY src ./src/
COPY templates ./templates/
COPY Cargo.toml .

RUN source $HOME/.cargo/env && cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

WORKDIR /app

RUN apk update
RUN apk upgrade
RUN apk add ca-certificates

EXPOSE 8000

COPY static .
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/trangarcom .

CMD ["./trangarcom"]

