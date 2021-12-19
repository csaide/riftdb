# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: GPL-3.0

FROM --platform=${BUILDPLATFORM} rust:1-bullseye AS build

WORKDIR /opt/riftdb

ARG TARGETPLATFORM

ENV BUILD_DEPS "upx-ucl"
ENV PROBE_VERSION "v0.4.6"
RUN apt-get update \
    && apt-get upgrade -yqq \
    && case "${TARGETPLATFORM}" in \
    "linux/arm64") \
    apt-get install -yqq gcc-aarch64-linux-gnu ${BUILD_DEPS} \
    && rustup target add aarch64-unknown-linux-gnu \
    && curl -L https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${PROBE_VERSION}/grpc_health_probe-linux-arm64 --output grpc_health_probe ;; \
    "linux/amd64") \
    apt-get install -yqq gcc ${BUILD_DEPS} \
    && rustup target add x86_64-unknown-linux-gnu \
    && curl -L https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${PROBE_VERSION}/grpc_health_probe-linux-amd64 --output grpc_health_probe ;; \
    *) exit 1 ;; \
    esac \
    && rustup component add rustfmt \
    && rm -rf /var/lib/apt/lists/*

# Copy in the Cargo configuration to fetch deps.
COPY Cargo.toml Cargo.lock .cargo/ ./
RUN cargo fetch

# Now copy in the source code to build and then compress the binary.
COPY . .

ARG TARGETARCH
ARG BUILD=debug

RUN --mount=type=cache,target=/root/.cargo \
    --mount=type=cache,sharing=private,target=/opt/riftdb/target \
    make BUILD=${BUILD} BUILD_ARCH=${TARGETARCH} compile

FROM --platform=${TARGETPLATFORM} gcr.io/distroless/static:latest AS runtime

ARG TARGETARCH
ARG BUILD=debug

COPY --from=build --chown=nonroot:nonroot --chmod=0700 /opt/riftdb/grpc_health_probe /bin/grpc_health_probe
COPY --from=build --chown=nonroot:nonroot --chmod=0700 /opt/riftdb/output/${BUILD}/riftd_linux-${TARGETARCH} /bin/riftd
COPY --from=build --chown=nonroot:nonroot --chmod=0700 /opt/riftdb/output/${BUILD}/riftctl_linux-${TARGETARCH} /bin/riftctl

LABEL org.opencontainers.image.source=https://github.com/csaide/riftdb

HEALTHCHECK --interval=10s --timeout=1s --start-period=5s --retries=3 CMD [ "/bin/grpc_health_probe", "-addr", "localhost:8081" ]

USER nonroot
VOLUME [ "/var/lib/riftd" ]
ENTRYPOINT [ "/bin/riftd" ]
CMD [ "--help" ]
