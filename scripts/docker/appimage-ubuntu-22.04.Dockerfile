FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH=/root/.cargo/bin:${PATH}
ENV OCCT_ROOT=/opt/opencascade
ENV LD_LIBRARY_PATH=/opt/opencascade/lib

# AppImage build SDK. Graphics/window loaders remain host libraries. The
# package runs on distributions whose glibc is at least the build
# system's: building on Ubuntu 22.04 (glibc 2.35) covers Debian 12, Ubuntu
# 22.04 and later. Ubuntu 22.04 does not ship OCCT 7.9, so it is built from
# pinned source into /opt/opencascade. The Debian package is built with
# scripts/docker/ubuntu-26.04.Dockerfile against Ubuntu's OCCT instead.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        dbus-x11 \
        desktop-file-utils \
        file \
        git \
        libdbus-1-3 \
        libfontconfig-dev \
        libfreetype-dev \
        libfuse2 \
        libssl-dev \
        libudev-dev \
        libvulkan-dev \
        libwayland-dev \
        libx11-dev \
        libx11-xcb1 \
        libxcursor1 \
        libxi6 \
        libxkbcommon-dev \
        libxkbcommon-x11-dev \
        pkg-config \
        mesa-vulkan-drivers \
        ninja-build \
        patchelf \
        squashfs-tools \
        vulkan-tools \
        xauth \
        xclip \
        xdg-utils \
        xdotool \
        xvfb \
        xz-utils \
        zenity \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain stable

# Copy only the Rust task's workspace inputs. No frontend, git history or user files.
COPY Cargo.toml Cargo.lock rust-toolchain.toml VERSION /tmp/nbcad-build-tools/
COPY crates /tmp/nbcad-build-tools/crates/
COPY xtask /tmp/nbcad-build-tools/xtask/
COPY src/i18n/en.json src/i18n/zh-CN.json src/i18n/es.json src/i18n/de.json /tmp/nbcad-build-tools/src/i18n/
WORKDIR /tmp/nbcad-build-tools
# Optional --build-arg to cap OCCT compile jobs on a shared machine.
ARG CMAKE_BUILD_PARALLEL_LEVEL
RUN cargo run --quiet --locked --manifest-path /tmp/nbcad-build-tools/Cargo.toml \
      -p xtask -- build-occt --prefix /opt/opencascade \
    && rm -rf /tmp/nbcad-build-tools

WORKDIR /workspace
