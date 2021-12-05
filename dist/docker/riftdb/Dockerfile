# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: GPL-3.0

FROM --platform=${BUILDPLATFORM} rust:1-bullseye AS build

WORKDIR /opt/riftdb

ARG TARGETPLATFORM

ENV BUILD_DEPS "upx-ucl"
RUN apt-get update \
    && apt-get upgrade -yqq \
    && case "${TARGETPLATFORM}" in \
    "linux/arm64") \
    apt-get install -yqq gcc-aarch64-linux-gnu ${BUILD_DEPS} \
    && rustup target add aarch64-unknown-linux-gnu ;; \
    "linux/amd64") \
    apt-get install -yqq gcc ${BUILD_DEPS} \
    && rustup target add x86_64-unknown-linux-gnu ;; \
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

RUN make BUILD=${BUILD} BUILD_ARCH=${TARGETARCH} compile

FROM --platform=${TARGETPLATFORM} gcr.io/distroless/static AS runtime

ARG TARGETARCH
ARG BUILD=debug

COPY --from=build --chown=nonroot:nonroot --chmod=0700 /opt/riftdb/target/output/${BUILD}/riftd_linux-${TARGETARCH} /bin/riftd
COPY --from=build --chown=nonroot:nonroot --chmod=0700 /opt/riftdb/target/output/${BUILD}/riftctl_linux-${TARGETARCH} /bin/riftctl

LABEL org.opencontainers.image.source=https://github.com/csaide/riftdb

USER nonroot
VOLUME [ "/var/lib/riftd" ]
ENTRYPOINT [ "/bin/riftd" ]
CMD [ "--help" ]
