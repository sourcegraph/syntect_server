########################################
# Rust nightly + musl in a build stage #
########################################
# Select specific Rust nightly version
FROM rust:1.32@sha256:741edb658fac7ac8a978bb30f83fb5d3a7b8e8fc35105a79f424b5671cca724a as our-rust-nightly
RUN rustup default nightly-2018-11-29

# Install musl compiler toolchain
RUN apt-get -y update && apt-get install --no-install-recommends -y musl-tools=1.1.16-3
RUN rustup target add x86_64-unknown-linux-musl

###################################
# Build syntect_server statically #
###################################
FROM our-rust-nightly as ss
COPY . /repo
WORKDIR /repo
RUN env 'CC_x86_64-unknown-linux-musl=musl-gcc' cargo rustc --release --target x86_64-unknown-linux-musl -- -C 'linker=musl-gcc'
RUN cp ./target/x86_64-unknown-linux-musl/release/syntect_server /syntect_server

#######################
# Compile final image #
#######################
FROM sourcegraph/alpine:3.9@sha256:e9264d4748e16de961a2b973cc12259dee1d33473633beccb1dfb8a0e62c6459
COPY --from=ss syntect_server /

# Use tini (https://github.com/krallin/tini) for proper signal handling.
RUN apk add --no-cache tini=0.18.0-r0
ENTRYPOINT ["/sbin/tini", "--"]

EXPOSE 9238
ENV ROCKET_ENV "production"
ENV ROCKET_PORT 9238
ENV ROCKET_LIMITS "{json=10485760}"

# syntect_server does not need a secret key since it uses no cookies, but
# without one set Rocket emits a warning.
ENV ROCKET_SECRET_KEY "+SecretKeyIsIrrelevantAndUnusedPleaseIgnore="

# When keep-alive is on, we observe connection resets in our Go clients of
# syntect_server. It is unclear why this is, especially because our Go clients do
# not reuse the connection (i.e. we make a fresh connection every time).
# Disabling keep-alive does resolve the issue though, our best guess is that
# this is a bug in Hyper 0.10 (see https://github.com/SergioBenitez/Rocket/issues/928#issuecomment-464632953).
# See https://github.com/sourcegraph/sourcegraph/issues/2615 for details on
# what we observed when this was enabled with the default 5s.
ENV ROCKET_KEEP_ALIVE=0

CMD ["/syntect_server"]
