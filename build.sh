env 'CC_x86_64-unknown-linux-musl=x86_64-linux-musl-gcc' cargo build --release --target x86_64-unknown-linux-musl
docker build -t sourcegraph/syntect_server .
