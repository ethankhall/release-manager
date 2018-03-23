FROM archlinux/base

RUN pacman -Syu --noconfirm && pacman -S --noconfirm wget base-devel clang git openssh cmake
RUN mkdir /tmp/pkg && \
    wget -q --directory-prefix=/tmp/pkg https://dl.bintray.com/ethankhall/generic/packages/osxcross-git-0.14-1-x86_64.pkg.tar.xz && \
    pacman -U --noconfirm /tmp/pkg/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
COPY cargo-config /root/.cargo/config

RUN /root/.cargo/bin/rustup target add x86_64-apple-darwin

ENV PATH=$PATH:/root/.cargo/bin/:/usr/local/osx-ndk-x86/bin