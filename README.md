# ash-2dgame-template

## What's this?

Rustのashを使った2Dゲーム向けのテンプレートコードです。

> [!WARNING]
> マルチスレッドに対応していません。
> マルチスレッドで動作させる場合は多くの箇所に修正が必要です。

## Build

次を用意してください:

- Git
- Cargo
- Python
- CMake
- Ninja
- MSVC環境 (Windows)
- Xcode CLT (macOS)

次を実行してください:

```sh
# Debugビルド
cargo build

# Releaseビルド
cargo build --release
```

> [!NOTE]
> 依存パッケージのインストールにそれなりの時間がかかります。

> [!NOTE]
> Vulkan Validation Layersはデバッグ時かつ`vvl` feature有効時のみ利用できます。
> 1. `cargo build --features vvl`でビルドし、
> 2. `cargo run --features vvl`で実行してください。

## Modules

```mermaid
classDiagram
    class Entry
    class Context
    class Submitter
    class Descriptors
    class RenderPass
    class Window
    class Swapchain
    class Framebuffer
    class InputStates
    class Game

    Entry "1" -- "0..*" Context
    Context "1" -- "0..*" Submitter
    Context "1" -- "0..*" Descriptors
    Context "1" -- "0..*" RenderPass
    Context "1" -- "0..*" Framebuffer
    Context "1" -- "0..*" Swapchain
    Window "1" -- "0,1" Swapchain
    Swapchain "0,1" -- "0..*" Framebuffer
    Descriptors "0,1" -- "0..*" RenderPass
    Framebuffer "1..*" -- "1" RenderPass
    Window "1..*" -- "0..*" InputStates
    Game "0..*" -- "1..*" InputStates
```
