# Build Linux binaries against Debian 12's glibc instead of the rolling host.
FROM node:24-bookworm-slim AS node
FROM rust:1.93.1-bookworm
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
 && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
    build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev \
    libudev-dev libayatana-appindicator3-dev dpkg-dev rpm \
    ca-certificates python3 git file binutils xz-utils \
    xvfb xauth dbus-x11 fonts-liberation desktop-file-utils \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /work
ENV AULA_DISABLE_HARDWARE_WRITES=1 CARGO_TERM_COLOR=never
