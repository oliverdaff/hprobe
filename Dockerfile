## Use cross build --target x86_64-unknown-linux-musl --release to build the binary.
FROM rust:1.45.0 AS build
COPY target/x86_64-unknown-linux-musl/release/hprobe .
RUN strip hprobe

FROM scratch
COPY  --from=build hprobe .
USER 1000
ENTRYPOINT ["./hprobe"]