# 构建说明

此项目使用 Dev Container 来确保在不同机器上的一致构建环境。

## 先决条件

- Docker
- 带有 "Dev Containers" 扩展的 VS Code

## 设置

1. 在 VS Code 中打开项目。
2. 当提示时点击 "Reopen in Container"，或运行命令 `Dev Containers: Reopen in Container`。
3. 等待容器构建并初始化。

## 构建产物

在 VS Code 终端内运行以下命令（该终端在容器内运行）。

### 1. Linux (AppImage & RPM)

```bash
npm run tauri build -- --bundles appimage,rpm
```

产物将在 `src-tauri/target/release/bundle/` 中。

### 2. Windows (EXE)

我们使用 `cargo-xwin` 进行交叉编译。

```bash
npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
```

产物将在 `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/` 中。

### 3. Android (APK)

为 arm64-v8a 构建。

```bash
npm run tauri android build -- --apk --target aarch64-linux-android
```

产物将在 `src-tauri/gen/android/app/build/outputs/apk/` 中。

## 注意事项

- **Android SDK**：容器包含 Android SDK 35 和 NDK 29.0.14206865。
- **Windows**：交叉编译使用 `cargo-xwin` 和 `nsis`。
- **一致性**：所有构建都在 `.devcontainer/Dockerfile` 中定义的相同 Docker 环境中运行。
