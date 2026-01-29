import Cocoa
import QuartzCore

class Window: NSObject, NSWindowDelegate {
  // NOTE: Cocoaでは装飾キーのキーコードが用意されていないので独自に追加する。
  enum ExtKeyCode {
    static let command: UInt16 = 999
  }

  let window: NSWindow
  let metalLayer: CAMetalLayer
  let frame: NSRect
  var running: Bool = true
  var inputStates: [UInt16: Bool] = [:]

  init?(title: String, width: UInt32, height: UInt32) {
    window = NSWindow(
      contentRect: NSRect(x: 0, y: 0, width: Int(width), height: Int(height)),
      styleMask: [.titled, .closable, .miniaturizable],
      backing: .buffered,
      defer: false
    )

    window.title = title
    window.center()
    window.makeKeyAndOrderFront(nil)
    window.collectionBehavior = .fullScreenPrimary

    guard let view = window.contentView else {
      window.close()
      return nil
    }

    metalLayer = CAMetalLayer()
    view.wantsLayer = true
    view.layer = metalLayer

    frame = window.frame

    super.init()

    window.delegate = self
  }

  deinit {
    window.delegate = nil
    window.close()
  }

  func processEvents() -> Bool {
    let app = NSApplication.shared

    while let event = app.nextEvent(matching: .any, until: nil, inMode: .default, dequeue: true) {
      guard event.window == window else {
        app.sendEvent(event)
        continue
      }

      switch event.type {
      case .keyDown:
        inputStates[event.keyCode] = true
      case .keyUp:
        inputStates[event.keyCode] = false
      case .flagsChanged:
        inputStates[ExtKeyCode.command] = event.modifierFlags.contains(.command)
      default:
        break
      }

      app.sendEvent(event)
    }

    return running
  }

  func toggleFullScreen() {
    var finished = false

    let enterObserver = NotificationCenter.default.addObserver(
      forName: NSWindow.didEnterFullScreenNotification,
      object: window,
      queue: nil
    ) { _ in
      finished = true
    }
    let exitObserver = NotificationCenter.default.addObserver(
      forName: NSWindow.didExitFullScreenNotification,
      object: window,
      queue: nil
    ) { _ in
      finished = true
    }

    window.toggleFullScreen(nil)

    while !finished {
      RunLoop.current.run(mode: .default, before: Date(timeIntervalSinceNow: 0.01))
    }

    NotificationCenter.default.removeObserver(enterObserver)
    NotificationCenter.default.removeObserver(exitObserver)
  }

  // NOTE: 非resizableなウィンドウはフルスクリーン時にもコンテントサイズを保つ。
  //       それでは困るので、このデリゲートを実装して、フルスクリーン時のコンテントサイズをスクリーンサイズにする。
  func window(_ window: NSWindow, willUseFullScreenContentSize proposedSize: NSSize) -> NSSize {
    if let screen = window.screen {
      return screen.frame.size
    }
    return proposedSize
  }

  // NOTE: `window(_:willUseFullScreenContentSize:)`を伴ってフルスクリーンになった後
  //       ウィンドウに戻るとコンテントサイズは維持される。
  //       それでは困るので、フルスクリーン終了直前にウィンドウフレームをウィンドウ作成時のものに差し替える。
  func windowWillExitFullScreen(_ notification: Notification) {
    window.setFrame(frame, display: false)
  }

  func windowWillClose(_ notification: Notification) {
    running = false
  }
}
