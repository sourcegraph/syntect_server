FROM alpine:latest
ADD ./target/x86_64-unknown-linux-musl/release/syntect_server /
EXPOSE 80
ENV ROCKET_ENV "production"
CMD ["/syntect_server"]
