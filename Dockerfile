ARG BINARY_NAME_DEFAULT=url-shortener

FROM clux/muslrust:stable AS builder
RUN groupadd -g 10001 -r dockergrp && useradd -r -g dockergrp -u 10001 dockeruser

ARG BINARY_NAME_DEFAULT
ENV BINARY_NAME=$BINARY_NAME_DEFAULT

COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/url_shortener*

COPY src ./src
RUN cargo build --target x86_64-unknown-linux-musl --release && \
    cp target/x86_64-unknown-linux-musl/release/url-shortener /url-shortener

FROM scratch

ARG BINARY_NAME_DEFAULT
ENV BINARY_NAME=$BINARY_NAME_DEFAULT

COPY --from=builder /etc/passwd /etc/passwd

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=builder /url-shortener /url-shortener

USER dockeruser
EXPOSE 3000
ENTRYPOINT ["/url-shortener"]