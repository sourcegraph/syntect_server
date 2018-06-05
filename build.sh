set -ex
env 'CC_x86_64-unknown-linux-musl=x86_64-linux-musl-gcc' cargo rustc --release --target x86_64-unknown-linux-musl -- -C 'linker=x86_64-linux-musl-gcc'
docker build --no-cache -t sourcegraph/syntect_server .
