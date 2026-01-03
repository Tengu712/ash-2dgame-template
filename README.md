# ash-2dgame-template

## What's this?

Rustのashを使った2Dゲーム向けのテンプレートコードです。

> [!WARNING]
> マルチスレッドに対応していません。
> マルチスレッドで動作させる場合は多くの箇所に修正が必要です。

## Build

次を用意してください:

- Cargo
- Conan2あるいはuv
- Ninja
- MSVC環境 (Windows)

次を実行してください:

```
cargo build
```

> [!NOTE]
> デバッグビルド時のみVulkan Validation Layersをインストールします。
> このインストールにはそこそこの時間がかかります。

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

    Entry "1" -- "0..*" Context
    Context "1" -- "0..*" Submitter
    Context "1" -- "0..*" Descriptors
    Context "1" -- "0..*" RenderPass
    Context "1" -- "0..*" Swapchain
    Context "1" -- "0..*" Framebuffer
    Window "1" -- "1" Swapchain
    Swapchain "0,1" -- "0..*" Framebuffer
    Descriptors "0,1" -- "0..*" RenderPass
    Framebuffer "1..*" -- "1" RenderPass
```
