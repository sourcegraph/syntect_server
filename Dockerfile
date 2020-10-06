########################################
# Rust nightly + musl in a build stage #
########################################
# Select specific Rust nightly version
FROM rust:1.46.0@sha256:90d7ddc83ab195dbe2d2f56e94c0a3f47952fcc1990d6d9b14d6d4213333a43e as our-rust-builder

# Install musl compiler toolchain
RUN apt-get -y update && apt-get install --no-install-recommends -y musl-tools=1.1.21-2 clang=1:7.0-47 llvm=1:7.0-47
RUN rustup target add x86_64-unknown-linux-musl

###################################
# Build syntect_server statically #
###################################
FROM our-rust-builder as ss
COPY . /repo
WORKDIR /repo
RUN env 'CC_x86_64-unknown-linux-musl=musl-gcc' cargo rustc --release --target x86_64-unknown-linux-musl -- -C 'linker=musl-gcc'
RUN cp ./target/x86_64-unknown-linux-musl/release/syntect_server /syntect_server

################################
# Build http-server-stabilizer #
################################
FROM golang:1.15.0-alpine@sha256:73182a0a24a1534e31ad9cc9e3a4bb46bb030a883b26eda0a87060f679b83607 as hss

RUN apk add --no-cache git=2.26.2-r0
RUN git clone https://github.com/slimsag/http-server-stabilizer /repo
WORKDIR /repo
RUN git checkout v1.0.4 && go build -o /http-server-stabilizer .

#######################
# Compile final image #
#######################
FROM sourcegraph/alpine:3.12@sha256:133a0a767b836cf86a011101995641cf1b5cbefb3dd212d78d7be145adde636d
COPY --from=ss syntect_server /
COPY --from=hss http-server-stabilizer /

EXPOSE 9238
ENV ROCKET_ENV "production"
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

# The more workers, the more resilient syntect_server is to getting stuck on
# bad grammar/file combinations. If it happens with four workers, only 1/4th of
# requests will be affected for a short period of time. Each worker can require
# at peak around 1.1 GiB of memory.
ENV WORKERS=4

ENV QUIET=true
CMD ["sh", "-c", "/http-server-stabilizer -listen=:9238 -prometheus-app-name=syntect_server -workers=$WORKERS -- env ROCKET_PORT={{.Port}} /syntect_server"]
