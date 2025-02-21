FROM --platform=$BUILDPLATFORM rust:1.84.0 AS cross
ARG TARGETARCH
COPY install-prerequisites.sh /tmp/install-prerequisites.sh
RUN /tmp/install-prerequisites.sh


COPY / /src
WORKDIR /src

RUN cargo fetch
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64)   rustArch='x86_64-unknown-linux-gnu'        ;; \
        arm)     rustArch='armv7-unknown-linux-gnueabihf'   ;; \
        arm64)   rustArch='aarch64-unknown-linux-gnu'       ;; \
        ppc64el) rustArch='powerpc64le-unknown-linux-gnu' ;; \
        s390x)   rustArch='s390x-unknown-linux-gnu'         ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    cargo build --release --target $rustArch && \
    mkdir /dist && \
    mv ./target/$rustArch/release/fix-kube-forwarder /dist

FROM --platform=$TARGETPLATFORM debian AS final
COPY --from=cross /dist /opt/fix-proxy
CMD ["/opt/fix-proxy/fix-kube-forwarder"]
