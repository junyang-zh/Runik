FROM ubuntu:jammy

ARG QEMU_VERSION=7.2.0
ARG HOME=/root

# 0. Install general tools
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y curl git python3 wget xz-utils unzip

# 1. Set up QEMU RISC-V
# - https://www.qemu.org/download/
# - https://fossies.org/linux/misc/ (alternate source)

# 1.1. Download source
WORKDIR ${HOME}
RUN wget https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz && \
    tar -xvf qemu-${QEMU_VERSION}.tar.xz

# 1.2. Install dependencies
# - https://risc-v-getting-started-guide.readthedocs.io/en/latest/linux-qemu.html#prerequisites
RUN apt-get install -y \
        autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
        gawk build-essential bison flex texinfo gperf libtool patchutils bc \
        zlib1g-dev libexpat-dev python3.10-dev \
        ninja-build pkg-config libglib2.0-dev libpixman-1-dev libsdl2-dev

# 1.3. Build and install from source
WORKDIR ${HOME}/qemu-${QEMU_VERSION}
RUN ./configure --target-list=riscv64-softmmu,riscv64-linux-user && \
    make -j$(nproc) && \
    make install

# 1.4. Clean up
WORKDIR ${HOME}
RUN rm -rf qemu-${QEMU_VERSION} qemu-${QEMU_VERSION}.tar.xz

# 1.5. Sanity checking
RUN qemu-system-riscv64 --version && \
    qemu-riscv64 --version

# 2. Set up Rust
# - https://www.rust-lang.org/tools/install

# 2.1. Install
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly
RUN set -eux; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

# 2.2. Sanity checking
RUN rustup --version && \
    cargo --version && \
    rustc --version

# 2.3. Rust src mirror
RUN CARGO_CONF=$CARGO_HOME'/config'; \
    BASHRC='/root/.bashrc' \
    && echo 'export RUSTUP_DIST_SERVER=https://rsproxy.cn' >> $BASHRC \
    && echo 'export RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup' >> $BASHRC \
    && touch $CARGO_CONF \
    && echo '[source.crates-io]' > $CARGO_CONF \
    && echo "replace-with = 'rsproxy-sparse'" >> $CARGO_CONF \
    && echo '[source.rsproxy]' >> $CARGO_CONF \
    && echo 'registry = "https://rsproxy.cn/crates.io-index"' >> $CARGO_CONF \
    && echo '[source.rsproxy-sparse]' >> $CARGO_CONF \
    && echo 'registry = "sparse+https://rsproxy.cn/index/"' >> $CARGO_CONF \
    && echo '[registries.rsproxy]' >> $CARGO_CONF \
    && echo 'index = "https://rsproxy.cn/crates.io-index"' >> $CARGO_CONF \
    && echo '[net]' >> $CARGO_CONF \
    && echo 'git-fetch-with-cli = true' >> $CARGO_CONF

# 2.4. Build environment
RUN rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils && \
    rustup component add rust-src && \
    rustup component add llvm-tools-preview

# 3. Download and add RISCV Toolchains (gdb, objdump, etc)
WORKDIR /usr/local/
ARG RV_TOOLCHAINS_HREF=https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2023.04.29
ARG RV_TOOLCHAINS="riscv64-elf-ubuntu-22.04-nightly-2023.04.29-nightly riscv64-glibc-ubuntu-22.04-nightly-2023.04.29-nightly"
RUN for RV_TOOLCHAIN in ${RV_TOOLCHAINS}; do \
        wget ${RV_TOOLCHAINS_HREF}/${RV_TOOLCHAIN}.tar.gz && \
        tar -xvf ${RV_TOOLCHAIN}.tar.gz && \
        cp -r riscv/* . && \
        rm -f ${RV_TOOLCHAIN}.tar.gz && \
        rm -rf riscv; \
    done
WORKDIR ${HOME}

# 4. Install llvm-clang
RUN apt-get install -y llvm clang lld

# 5. Clean up apt caches
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

# Ready to go
WORKDIR ${HOME}