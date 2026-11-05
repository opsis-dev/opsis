FROM ubuntu:26.04 AS tools

ARG SAMPLY_VERSION
ARG SAMPLY_SHA256

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    xvfb xauth dbus dbus-x11 gnome-keyring libcap2-bin procps ca-certificates curl xz-utils time \
    heaptrack linux-perf fonts-dejavu-core \
    libnss3 libnspr4 libgbm1 libasound2t64 libxkbcommon0 libxcomposite1 libxdamage1 \
    libxrandr2 libatk-bridge2.0-0t64 libcups2t64 libpango-1.0-0 libcairo2 \
    libgl1-mesa-dri libegl1 libegl-mesa0 libgles2 libvulkan1 mesa-vulkan-drivers \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSLo /tmp/samply.tar.xz \
    "https://github.com/mstange/samply/releases/download/samply-v${SAMPLY_VERSION}/samply-x86_64-unknown-linux-gnu.tar.xz" \
 && echo "${SAMPLY_SHA256}  /tmp/samply.tar.xz" | sha256sum -c - \
 && mkdir /tmp/samply && tar -xJf /tmp/samply.tar.xz -C /tmp/samply \
 && find /tmp/samply -type f -name samply -exec install -m 0755 {} /usr/local/bin/samply \; \
 && rm -rf /tmp/samply /tmp/samply.tar.xz \
 && setcap cap_perfmon,cap_ipc_lock+ep /usr/local/bin/samply \
 && setcap cap_perfmon,cap_ipc_lock+ep /usr/bin/perf

FROM tools

COPY opsis.deb /tmp/opsis.deb

RUN apt-get update \
 && apt-get install -y --no-install-recommends /tmp/opsis.deb \
 && rm -rf /var/lib/apt/lists/* /tmp/opsis.deb
