ARG BUILDER_ARCH=armv7-musleabihf
ARG TARGET_ARCH=armv7-unknown-linux-musleabihf

FROM messense/rust-musl-cross:${BUILDER_ARCH} AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG TARGET_ARCH
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target=${TARGET_ARCH} --recipe-path recipe.json
COPY . .
RUN cargo build --release --target=${TARGET_ARCH}
RUN musl-strip -s /app/target/${TARGET_ARCH}/release/home-consule

FROM scratch AS runtime
ARG VERSION
ARG BUILD_DATE
ARG TARGET_ARCH
LABEL version="${VERSION}" \
    description="Containerized home-consule: A dashboard for your homelab integrated with Consul" \
    org.label-schema.schema-version="1.0" \
    org.label-schema.name="zerotier" \
    org.label-schema.description="Containerized home-consule: A dashboard for your homelab integrated with Consul" \
    org.label-schema.build-date="${BUILD_DATE}" \
    org.label-schema.url="https://github.com/bltavares/home-consule" \
    org.label-schema.version="{$VERSION}" \
    org.label-schema.docker.cmd="docker run -d \
    --restart=unless-stopped \
    -v \${PWD}:/app \
    -p 3000:3000 \
    -e CONSUL_HTTP_TOKEN \
    -e CONSUL_HTTP_ADDR \
    --name home \
    bltavares/home-consule"
COPY --from=builder /app/target/${TARGET_ARCH}/release/home-consule /usr/local/bin/
WORKDIR /app
ENTRYPOINT ["/usr/local/bin/home-consule"]
