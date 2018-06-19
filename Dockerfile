FROM alpine:latest

# Use tini (https://github.com/krallin/tini) for proper signal handling.
RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

ADD ./target/x86_64-unknown-linux-musl/release/syntect_server /
EXPOSE 9238
ENV ROCKET_ENV "production"
ENV ROCKET_PORT 9238
ENV ROCKET_LIMITS "{json=10485760}"

# syntect_server does not need a secret key since it uses no cookies, but
# without one set Rocket emits a warning.
ENV ROCKET_SECRET_KEY "+SecretKeyIsIrrelevantAndUnusedPleaseIgnore="

RUN addgroup -S sourcegraph && adduser -S -G sourcegraph -h /home/sourcegraph sourcegraph
USER sourcegraph

CMD ["/syntect_server"]
