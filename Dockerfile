FROM alpine:latest

# Use tini (https://github.com/krallin/tini) for proper signal handling.
RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

ADD ./target/x86_64-unknown-linux-musl/release/syntect_server /
EXPOSE 80
ENV ROCKET_ENV "production"
CMD ["/syntect_server"]
