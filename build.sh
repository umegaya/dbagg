docker run --rm -ti -v `pwd`:/build -v $HOME/.multirust/toolchains/nightly/cargo:/root/.cargo umegaya/rustatic bash -c "cd /build && cargo build --target=x86_64-unknown-linux-musl"
docker build -t umegaya/dbagg .
docker push umegaya/dbagg
