FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates clang curl git python3 xz-utils unzip \
 && rm -rf /var/lib/apt/lists/*
