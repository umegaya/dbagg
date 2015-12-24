FROM alpine:latest

ADD target/x86_64-unknown-linux-musl/debug/dbagg /usr/local/bin/dbagg

