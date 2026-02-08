// macOSにおけるウィンドウライブラリ
//
// NOTE: Rustにはお手軽なウィンドウライブラリがないため、仕方なくSwiftで書いている。
//
// WARN: マルチスレッド対応をしていないため、必ず単一スレッドで動作すること。

import Cocoa

var initialized = false
var window: Window? = nil

func ensureInitialized() {
  guard !initialized else {
    return
  }
  NSApplication.shared.setActivationPolicy(.regular)
  initialized = true
}

@_cdecl("show_error_dialog")
func show_error_dialog(message: UnsafePointer<CChar>) {
  ensureInitialized()
  let alert = NSAlert()
  alert.messageText = "ERROR"
  alert.informativeText = String(cString: message)
  alert.alertStyle = .critical
  alert.addButton(withTitle: "OK")
  alert.runModal()
}

@_cdecl("get_metal_layer")
func get_metal_layer() -> UnsafeMutableRawPointer? {
  ensureInitialized()
  return window.map { Unmanaged.passUnretained($0.metalLayer).toOpaque() }
}

@_cdecl("create_window")
func create_window(title: UnsafePointer<CChar>, width: UInt32, height: UInt32) -> UInt8 {
  ensureInitialized()
  guard window == nil else {
    return 0
  }
  window = Window(title: String(cString: title), width: width, height: height)
  return window != nil ? 1 : 0
}

@_cdecl("destroy_window")
func destroy_window() {
  ensureInitialized()
  window = nil
}

@_cdecl("process_window_events")
func process_window_events() -> UInt8 {
  ensureInitialized()
  return window?.processEvents() == true ? 1 : 0
}

@_cdecl("get_current_client_size")
func get_current_client_size() -> WindowSize {
  ensureInitialized()
  let contentRect = window?.window.contentView?.frame ?? .zero
  return WindowSize(
    width: UInt32(contentRect.width),
    height: UInt32(contentRect.height)
  )
}

@_cdecl("toggle_fullscreen")
func toggle_fullscreen() {
  ensureInitialized()
  window?.toggleFullScreen()
}

@_cdecl("get_input_state")
func get_input_state(code: UInt16) -> UInt8 {
  ensureInitialized()
  return window?.inputStates[code] == true ? 1 : 0
}

@_cdecl("get_scale_factor")
func get_scale_factor() -> Float {
  ensureInitialized()
  return Float(window?.window.backingScaleFactor ?? 1.0)
}
