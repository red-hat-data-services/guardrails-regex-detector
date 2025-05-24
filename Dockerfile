ARG UBI_MINIMAL_BASE_IMAGE=registry.access.redhat.com/ubi9/ubi-minimal
ARG UBI_BASE_IMAGE_TAG=latest

## Rust builder ################################################################
# Specific debian version so that compatible glibc version is used
FROM rust:1.87.0 AS rust-builder

WORKDIR /app

COPY rust-toolchain.toml rust-toolchain.toml

RUN rustup component add rustfmt

## Regex builder #########################################################
FROM rust-builder AS regex-detector-builder

COPY *.toml /app/
COPY src/ /app/src/

WORKDIR /app

RUN cargo install --root /app/ --path .

## Tests stage ##################################################################
FROM regex-detector-builder AS tests
RUN cargo test

## Lint stage ###################################################################
FROM regex-detector-builder AS lint
RUN cargo clippy --all-targets --all-features -- -D warnings

## Formatting check stage #######################################################
FROM regex-detector-builder AS format
RUN cargo fmt --check

## Release Image ################################################################

FROM ${UBI_MINIMAL_BASE_IMAGE}:${UBI_BASE_IMAGE_TAG} AS regex-detector-release

COPY --from=regex-detector-builder /app/bin/ /app/bin/

RUN microdnf install -y --disableplugin=subscription-manager shadow-utils compat-openssl11 && \
    microdnf clean all --disableplugin=subscription-manager

CMD ["/app/bin/regex-detector"]
