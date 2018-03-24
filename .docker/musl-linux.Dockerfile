FROM clux/muslrust

RUN apt-get update && apt-get install -y cmake
RUN cargo install rustfmt-nightly --force

COPY musl-build.sh /bin/musl-build.sh
RUN chmod +x /bin/musl-build.sh