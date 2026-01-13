//! Xcbにおけるウィンドウライブラリ
//!
//! NOTE: Rustにはお手軽なウィンドウライブラリがないため、仕方なくC++で書いている。
//!
//! WARN: マルチスレッド対応をしていないため、必ず単一スレッドで動作すること。

#include "Window.hpp"

#include <cstdlib>
#include <optional>
#include <xcb/xcb.h>

namespace {

struct ConnectionAndWindow {
	void *connection;
	uint32_t window;
};

xcb_connection_t *g_connection = nullptr;
xcb_screen_t *g_screen = nullptr;
std::optional<Window> g_window;

} // namespace

/// エラーメッセージダイアログを表示する関数
extern "C" void show_error_dialog(const char *message) {
	// TODO: unimplemented
}

/// XcbコネクションとウィンドウIDを取得する関数
///
/// 存在しない場合、0が返る。
extern "C" ConnectionAndWindow get_connection_and_window() {
	if (g_window && g_connection) {
		return {
			static_cast<void *>(g_connection),
			g_window->getWindow(),
		};
	} else {
		return {nullptr, 0};
	}
}

/// ウィンドウを作成する関数
///
/// 失敗時や既に作成済みの場合は0を返す。
extern "C" uint8_t create_window(const char *title, uint16_t width, uint16_t height) {
	if (g_window) {
		return 0;
	}

	if (!g_connection) {
		g_connection = xcb_connect(nullptr, nullptr);
		if (xcb_connection_has_error(g_connection)) {
			return 0;
		}
	}

	if (!g_screen) {
		const auto setup = xcb_get_setup(g_connection);
		const auto iter = xcb_setup_roots_iterator(setup);
		g_screen = iter.data;
	}

	g_window.emplace(g_connection, g_screen, title, width, height);
	return 1;
}

/// ウィンドウを閉じる関数
extern "C" void destroy_window() {
	if (g_window && g_connection) {
		g_window->destroy(g_connection);
		g_window.reset();
	}
}

/// 溜まったウィンドウイベントを処理する関数
///
/// ウィンドウが存在しない場合や・ウィンドウが閉じられた場合は0を返す。
extern "C" uint8_t process_window_events() {
	if (!g_window || !g_connection) {
		return 0;
	}

	xcb_generic_event_t *event;
	while ((event = xcb_poll_for_event(g_connection)) != nullptr) {
		switch (event->response_type & ~0x80) {
			case XCB_CLIENT_MESSAGE: {
				const auto e = reinterpret_cast<xcb_client_message_event_t *>(event);
				if (e->window == g_window->getWindow()) {
					if (g_window->shouldClose(e)) {
						return 0;
					}
				}
				break;
			}
			case XCB_KEY_PRESS: {
				const auto e = reinterpret_cast<xcb_key_press_event_t *>(event);
				if (e->event == g_window->getWindow()) {
					g_window->pressKey(e->detail);
				}
				break;
			}
			case XCB_KEY_RELEASE: {
				const auto e = reinterpret_cast<xcb_key_release_event_t *>(event);
				if (e->event == g_window->getWindow()) {
					g_window->releaseKey(e->detail);
				}
				break;
			}
		}
		free(event);
	}

	return 1;
}

/// 現在のウィンドウクライアントサイズを取得する関数
///
/// ウィンドウが存在しない場合、0x0のサイズを返す。
extern "C" WindowSize get_current_client_size() {
	return g_window && g_connection
		? g_window->getCurrentClientSize(g_connection)
		: WindowSize{0, 0};
}

/// フルスクリーン状態をトグルする関数
extern "C" void toggle_fullscreen() {
	if (g_window && g_connection && g_screen) {
		g_window->toggleFullscreen(g_connection, g_screen);
	}
}

/// 現在のキー入力状態を取得する関数
extern "C" uint8_t get_input_state(uint32_t code) {
	return g_window
		? static_cast<uint8_t>(g_window->getInputState(static_cast<xcb_keycode_t>(code)))
		: 0;
}
