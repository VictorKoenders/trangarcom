FROM ubuntu:19.04

RUN apt update && \
    apt upgrade -y && \
    apt install libssl-dev openssl libpq-dev -y

COPY target/release/trangarcom /trangarcom
COPY images /images
COPY static /static

ENTRYPOINT ["/trangarcom"]
