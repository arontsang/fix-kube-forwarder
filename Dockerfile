FROM --platform=$BUILDPLATFORM rust:1.84.0 AS cross
ARG TARGETARCH
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64)   rustArch='x86_64-unknown-linux-gnu'        ;; \
        arm)     rustArch='armv7-unknown-linux-gnueabihf'   ;; \
        arm64)   rustArch='aarch64-unknown-linux-gnu'       ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu' ;; \
        s390x)   rustArch='s390x-unknown-linux-gnu'         ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    rustup target add $rustArch

COPY / /src
WORKDIR /src

RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64)   rustArch='x86_64-unknown-linux-gnu'        ;; \
        arm)     rustArch='armv7-unknown-linux-gnueabihf'   ;; \
        arm64)   rustArch='aarch64-unknown-linux-gnu'       ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu' ;; \
        s390x)   rustArch='s390x-unknown-linux-gnu'         ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    case "${TARGETARCH}" in \
        amd64)    apt-get update && apt-get install gcc-x86-64-linux-gnu        -y ;; \
        arm)      apt-get update && apt-get install gcc-arm-linux-gnueabihf     -y ;; \
        arm64)    apt-get update && apt-get install gcc-aarch64-linux-gnu       -y ;; \
        ppc64el)  apt-get update && apt-get install gcc-powerpc64le-linux-gnu   -y ;; \
        s390x)    apt-get update && apt-get install gcc-s390x-linux-gnu         -y ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    cargo build --release --target-dir /dist --target $rustArch

FROM --platform=$TARGETPLATFORM debian AS final
COPY --from=cross /dist /opt/fix-proxy
