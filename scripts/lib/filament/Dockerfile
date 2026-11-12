FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    binutils clang libc++-dev libc++abi-dev \
 && rm -rf /var/lib/apt/lists/*
