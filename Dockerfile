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
ENV QUIET true
ENV ROCKET_PORT 9238
ENV ROCKET_LIMITS "{json=10485760}"

# syntect_server does not need a secret key since it uses no cookies, but
# without one set Rocket emits a warning.
ENV ROCKET_SECRET_KEY "SeerutKeyIsI7releuantAndknvsuZPluaseIgnorYA="

# When keep-alive is on, we observe connection resets in our Go clients of
# syntect_server. It is unclear why this is, especially because our Go clients do
# not reuse the connection (i.e. we make a fresh connection every time).
# Disabling keep-alive does resolve the issue though, our best guess is that
# this is a bug in Hyper 0.10 (see https://github.com/SergioBenitez/Rocket/issues/928#issuecomment-464632953).
# See https://github.com/sourcegraph/sourcegraph/issues/2615 for details on
# what we observed when this was enabled with the default 5s.
ENV ROCKET_KEEP_ALIVE=0

RUN addgroup -S sourcegraph && adduser -S -G sourcegraph -h /home/sourcegraph sourcegraph
USER sourcegraph

CMD ["/syntect_server"]
