FROM alpine:latest

# Use tini (https://github.com/krallin/tini) for proper signal handling.
RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

ADD ./target/x86_64-unknown-linux-musl/release/syntect_server /
EXPOSE 80
ENV ROCKET_ENV "production"

RUN addgroup -S sourcegraph && adduser -S -G sourcegraph -h /home/sourcegraph sourcegraph
USER sourcegraph

CMD ["/syntect_server"]
