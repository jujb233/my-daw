FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# 安装常见依赖和 Linux 构建依赖
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    build-essential \
    libasound2-dev \
    rpm \
    nsis \
    lld \
    llvm \
    clang \
    unzip \
    openjdk-21-jdk \
    sudo \
    libfuse2 \
    && rm -rf /var/lib/apt/lists/*

# 安装 Node.js 20
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs

# 设置 Rust 环境变量
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain stable

# 添加 Rust 目标
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-pc-windows-msvc \
    aarch64-linux-android \
    armv7-linux-androideabi \
    i686-linux-android \
    x86_64-linux-android

# 安装 cargo-xwin 用于 Windows 交叉编译
RUN cargo install cargo-xwin

# 安装 Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}"

# 下载命令行工具
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -O /tmp/cmdline-tools.zip && \
    unzip -q /tmp/cmdline-tools.zip -d ${ANDROID_HOME}/cmdline-tools && \
    mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

# 接受许可证并安装 SDK 组件
# 使用来自 build.yml 的 NDK 版本：29.0.14206865
RUN yes | sdkmanager --licenses && \
    sdkmanager "platforms;android-35" "build-tools;35.0.0" "ndk;29.0.14206865" "platform-tools"

# 设置 NDK 环境变量
ENV NDK_HOME=${ANDROID_HOME}/ndk/29.0.14206865
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/29.0.14206865

# 设置 Android 交叉编译的环境变量
ENV TOOLCHAIN=${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin
ENV CC_aarch64_linux_android="${TOOLCHAIN}/aarch64-linux-android26-clang"
ENV CXX_aarch64_linux_android="${TOOLCHAIN}/aarch64-linux-android26-clang++"
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${TOOLCHAIN}/aarch64-linux-android26-clang"
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C linker=${TOOLCHAIN}/aarch64-linux-android26-clang"

ENV CC_armv7_linux_androideabi="${TOOLCHAIN}/armv7a-linux-androideabi26-clang"
ENV CXX_armv7_linux_androideabi="${TOOLCHAIN}/armv7a-linux-androideabi26-clang++"
ENV CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="${TOOLCHAIN}/armv7a-linux-androideabi26-clang"
ENV CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUSTFLAGS="-C linker=${TOOLCHAIN}/armv7a-linux-androideabi26-clang"

ENV CC_i686_linux_android="${TOOLCHAIN}/i686-linux-android26-clang"
ENV CXX_i686_linux_android="${TOOLCHAIN}/i686-linux-android26-clang++"
ENV CARGO_TARGET_I686_LINUX_ANDROID_LINKER="${TOOLCHAIN}/i686-linux-android26-clang"
ENV CARGO_TARGET_I686_LINUX_ANDROID_RUSTFLAGS="-C linker=${TOOLCHAIN}/i686-linux-android26-clang"

ENV CC_x86_64_linux_android="${TOOLCHAIN}/x86_64-linux-android26-clang"
ENV CXX_x86_64_linux_android="${TOOLCHAIN}/x86_64-linux-android26-clang++"
ENV CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="${TOOLCHAIN}/x86_64-linux-android26-clang"
ENV CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-C linker=${TOOLCHAIN}/x86_64-linux-android26-clang"

# 修复非 root 用户的权限
RUN chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME} ${ANDROID_HOME}

# 允许 AppImage 工具在没有 FUSE 的情况下运行
ENV APPIMAGE_EXTRACT_AND_RUN=1

WORKDIR /workspace
