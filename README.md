# ash-2dgame-template

## What's this?

Rustのashを使った2Dゲーム向けのテンプレートコードです。

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

## Objects

```mermaid
classDiagram
    class Entry
    class Context
    class Window
    class Swapchain
    class Submitter

    Entry "1" -- "0..*" Context
    Context "1" -- "0..*" Submitter
    Context "1" -- "0..*" Swapchain
    Window "1" -- "1" Swapchain : same thread
```
