env 'CC_x86_64-unknown-linux-musl=musl-gcc' cargo rustc --release --target x86_64-unknown-linux-musl -- -C 'linker=musl-gcc'
docker build -t sourcegraph/syntect_server .
