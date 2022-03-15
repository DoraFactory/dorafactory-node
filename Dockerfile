FROM docker.io/paritytech/ci-linux:production as builder

ARG PROFILE=release
ARG BIN=dorafactory-node

WORKDIR /dorafactory

COPY . /dorafactory

RUN cargo build --$PROFILE --bin $BIN --features runtime-benchmarks --features try-runtime

# ===== SECOND STAGE ======

FROM docker.io/library/ubuntu:20.04

ARG PROFILE=release
ARG BIN=dorafactory-node

ENV BIN_PATH=/usr/local/bin/$BIN

COPY --from=builder /dorafactory/target/$PROFILE/$BIN /usr/local/bin

RUN apt update -y \
    && apt install -y ca-certificates libssl-dev \
    && useradd -m -u 1000 -U -s /bin/sh -d /dorafactory dorafactory \
    && mkdir -p /dorafactory/.local \
    && mkdir /data \
    && chown -R dorafactory:dorafactory /data \
    && ln -s /data /dorafactory/.local/share \
    && chown -R dorafactory:dorafactory /dorafactory/.local/share

USER dorafactory
WORKDIR /dorafactory
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/dorafactory/entrypoint.sh"]
