FROM ubuntu:22.04

# 1. 多源配置 - 设置多个备用源，确保下载可靠性
RUN sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \
    sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ jammy-backports main restricted universe multiverse" >> /etc/apt/sources.list

ENV DEBIAN_FRONTEND=noninteractive

# 2. 设置APT重试机制和超时时间
RUN echo 'Acquire::Retries "3";' > /etc/apt/apt.conf.d/80-retries && \
    echo 'Acquire::http::Timeout "120";' >> /etc/apt/apt.conf.d/80-retries && \
    echo 'Acquire::https::Timeout "120";' >> /etc/apt/apt.conf.d/80-retries

# 3. 安装常见依赖和 Linux 构建依赖（增加重试机制）
RUN apt-get update && \
    for i in {1..3}; do \
    apt-get install -y \
    curl \
    wget \
    ca-certificates \
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
    xz-utils \
    && break || sleep 10; \
    done && \
    rm -rf /var/lib/apt/lists/*

# 4. 安装 Node.js 22.20.0 (从淘宝镜像下载二进制包)
ENV NODE_VERSION=v22.20.0
ENV NODE_DISTRO=linux-x64
RUN for i in {1..3}; do \
    wget --no-check-certificate -O node-${NODE_VERSION}-${NODE_DISTRO}.tar.xz https://npmmirror.com/mirrors/node/${NODE_VERSION}/node-${NODE_VERSION}-${NODE_DISTRO}.tar.xz \
    && tar -xJf node-${NODE_VERSION}-${NODE_DISTRO}.tar.xz -C /usr/local --strip-components=1 \
    && rm node-${NODE_VERSION}-${NODE_DISTRO}.tar.xz \
    && /usr/local/bin/node -v \
    && /usr/local/bin/npm -v \
    && npm config set registry https://registry.npmmirror.com \
    && npm config set disturl https://npmmirror.com/dist \
    && break || sleep 10; \
    done

# 5. 设置 Rust 环境变量和镜像
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup \
    RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup

# 6. 安装 Rust（使用国内镜像）
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain stable

# 7. 配置 Cargo 镜像
RUN mkdir -p /usr/local/cargo && \
    echo '[source.crates-io]' > /usr/local/cargo/config && \
    echo "replace-with = 'tuna'" >> /usr/local/cargo/config && \
    echo '' >> /usr/local/cargo/config && \
    echo '[source.tuna]' >> /usr/local/cargo/config && \
    echo 'registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"' >> /usr/local/cargo/config && \
    echo '' >> /usr/local/cargo/config && \
    echo '[net]' >> /usr/local/cargo/config && \
    echo 'git-fetch-with-cli = true' >> /usr/local/cargo/config

# 8. 添加 Rust 目标（批量安装以加速，增加重试机制）
RUN for i in {1..3}; do \
    rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-pc-windows-msvc \
    aarch64-linux-android \
    armv7-linux-androideabi \
    i686-linux-android \
    x86_64-linux-android \
    && break || sleep 10; \
    done

# 9. 安装 cargo-xwin (使用 GitHub 镜像下载预编译二进制文件，避免编译耗时)
# 使用 mirror.ghproxy.com 加速 GitHub Release 下载
ENV CARGO_XWIN_VERSION=v0.18.3
RUN for i in {1..3}; do \
    wget -q --timeout=120 --tries=3 \
    https://mirror.ghproxy.com/https://github.com/rust-mobile/cargo-xwin/releases/download/${CARGO_XWIN_VERSION}/cargo-xwin-${CARGO_XWIN_VERSION}-x86_64-unknown-linux-musl.tar.gz \
    -O /tmp/cargo-xwin.tar.gz && \
    tar -xCz /tmp -f /tmp/cargo-xwin.tar.gz && \
    mv /tmp/cargo-xwin-${CARGO_XWIN_VERSION}-x86_64-unknown-linux-musl/cargo-xwin /usr/local/cargo/bin/cargo-xwin && \
    chmod +x /usr/local/cargo/bin/cargo-xwin && \
    rm -rf /tmp/cargo-xwin* && \
    break || sleep 10; \
    done

# 10. 安装 Android SDK（使用国内镜像）
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}"

# 创建 sdkmanager 配置文件，使用清华镜像
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools && \
    mkdir -p /root/.android && \
    echo '### User Sources for Android SDK Manager' > /root/.android/repositories.cfg && \
    echo 'count=0' >> /root/.android/repositories.cfg

# 下载命令行工具（增加重试机制）
RUN for i in {1..3}; do \
    wget -q --timeout=120 --tries=3 \
    https://googledownloads.cn/android/repository/commandlinetools-linux-11076708_latest.zip \
    -O /tmp/cmdline-tools.zip && break || sleep 30; \
    done && \
    unzip -q /tmp/cmdline-tools.zip -d ${ANDROID_HOME}/cmdline-tools && \
    mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

# 11. 配置 Android SDK 镜像源
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools/latest && \
    printf '#!/bin/bash\n\
    # Override sdkmanager to use mirror\n\
    ORIGINAL_SDKMANAGER=$(dirname "$0")/sdkmanager-original\n\
    if [ ! -f "$ORIGINAL_SDKMANAGER" ]; then\n\
    mv "$(dirname "$0")/sdkmanager" "$ORIGINAL_SDKMANAGER"\n\
    fi\n\
    \n\
    # Add mirror arguments\n\
    args=("$@")\n\
    mirror_args=()\n\
    for arg in "${args[@]}"; do\n\
    if [[ $arg == --no_https* ]]; then\n\
    mirror_args+=("$arg")\n\
    else\n\
    mirror_args+=("$arg")\n\
    # Add mirror repository\n\
    if [[ $arg == "sdkmanager" ]] || [[ $arg == *"install"* ]]; then\n\
    mirror_args+=("--channel=3")\n\
    mirror_args+=("--sdk_root=${ANDROID_SDK_ROOT:-$ANDROID_HOME}")\n\
    mirror_args+=("--verbose")\n\
    fi\n\
    fi\n\
    done\n\
    \n\
    # Set mirror repositories\n\
    export REPO_OS_OVERRIDE="linux"\n\
    export REPO_URL="https://googledownloads.cn/android/repository/"\n\
    \n\
    "$ORIGINAL_SDKMANAGER" "${mirror_args[@]}"\n' > ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager

RUN chmod +x ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager

# 12. 接受许可证并安装 SDK 组件（增加重试机制）
RUN yes | sdkmanager --licenses || true && \
    for i in {1..3}; do \
    sdkmanager \
    "platforms;android-35" \
    "build-tools;35.0.0" \
    "ndk;29.0.14206865" \
    "platform-tools" \
    --sdk_root=${ANDROID_HOME} \
    --channel=3 \
    --verbose \
    && break || sleep 30; \
    done

# 13. 设置 NDK 环境变量
ENV NDK_HOME=${ANDROID_HOME}/ndk/29.0.14206865
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/29.0.14206865

# 14. 设置 Android 交叉编译的环境变量
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

# 15. 修复非 root 用户的权限
RUN chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME} ${ANDROID_HOME}

# 16. 允许 AppImage 工具在没有 FUSE 的情况下运行
ENV APPIMAGE_EXTRACT_AND_RUN=1

# 17. 创建全局 npm 缓存配置
RUN npm config set cache /tmp/npm-cache --global && \
    npm config set registry https://registry.npmmirror.com --global && \
    npm config set sass_binary_site https://npmmirror.com/mirrors/node-sass/ --global && \
    npm config set phantomjs_cdnurl https://npmmirror.com/mirrors/phantomjs/ --global && \
    npm config set electron_mirror https://npmmirror.com/mirrors/electron/ --global && \
    npm config set puppeteer_download_host https://npmmirror.com/mirrors/chromium-browser-snapshots/ --global

# 18. 设置 Git 全局配置（避免超时）
RUN git config --global http.postBuffer 524288000 && \
    git config --global https.postBuffer 524288000 && \
    git config --global core.compression 0

# 19. 创建工作目录
WORKDIR /workspace

# 20. 创建用户友好的构建脚本
RUN printf '#!/bin/bash\n\
    # 带重试的构建脚本\n\
    MAX_RETRIES=3\n\
    RETRY_DELAY=10\n\
    \n\
    for i in $(seq 1 $MAX_RETRIES); do\n\
    echo "尝试 $i/$MAX_RETRIES: $@"\n\
    if $@; then\n\
    echo "构建成功"\n\
    exit 0\n\
    else\n\
    echo "构建失败，等待 ${RETRY_DELAY} 秒后重试..."\n\
    sleep $RETRY_DELAY\n\
    fi\n\
    done\n\
    \n\
    echo "构建失败，已达到最大重试次数"\n\
    exit 1\n' > /usr/local/bin/build-with-retry

RUN chmod +x /usr/local/bin/build-with-retry