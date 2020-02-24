ARG TON_TYPES_IMAGE=tonlabs/ton-labs-types:latest
ARG TON_BLOCK_IMAGE=tonlabs/ton-labs-block:latest

FROM alpine:latest as ton-labs-block-src
RUN addgroup --gid 1000 jenkins && \
    adduser -D -G jenkins jenkins
COPY --chown=jenkins:jenkins ./Cargo.* ./*.md ./*.rs /tonlabs/ton-labs-block/
COPY --chown=jenkins:jenkins ./src /tonlabs/ton-labs-block/src
WORKDIR /tonlabs/ton-labs-block
VOLUME /tonlabs/ton-labs-block
USER jenkins

FROM $TON_TYPES_IMAGE as ton-labs-types-src
FROM $TON_BLOCK_IMAGE as ton-labs-block-source
FROM alpine:latest as ton-labs-block-full
RUN addgroup --gid 1000 jenkins && \
    adduser -D -G jenkins jenkins
COPY --from=ton-labs-types-src --chown=jenkins:jenkins /tonlabs/ton-labs-types /tonlabs/ton-labs-types
COPY --from=ton-labs-block-source --chown=jenkins:jenkins /tonlabs/ton-labs-block /tonlabs/ton-labs-block
WORKDIR /tonlabs
VOLUME /tonlabs

FROM rust:latest as ton-labs-block-rust
RUN apt -qqy update && apt -qyy install apt-utils && \
    curl -sL https://deb.nodesource.com/setup_12.x | bash - && \
    apt-get install -qqy nodejs && \
    adduser --group jenkins && \
    adduser -q --disabled-password --gid 1000 jenkins && \
    mkdir /tonlabs && chown -R jenkins:jenkins /tonlabs
COPY --from=ton-labs-block-full --chown=jenkins:jenkins /tonlabs/ton-labs-types /tonlabs/ton-labs-types
COPY --from=ton-labs-block-full --chown=jenkins:jenkins /tonlabs/ton-labs-block /tonlabs/ton-labs-block
WORKDIR /tonlabs/ton-labs-block