# First, build our static binary.
#
# See https://github.com/emk/rust-musl-builder
FROM ekidd/rust-musl-builder:nightly AS builder
ADD . ./
RUN sudo chown -R rust:rust /home/rust
RUN cargo build --release

# Now build our actual Docker container using an alpine base image.
FROM alpine:latest

# Use tini (https://github.com/krallin/tini) for proper signal handling.
RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/syntect_server /
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
