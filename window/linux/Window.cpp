#include "Window.hpp"

#include <array>
#include <cstdlib>
#include <cstring>

namespace {

xcb_atom_t getAtom(xcb_connection_t *connection, const char *name) {
	const auto cookie = xcb_intern_atom(connection, 0, strlen(name), name);
	const auto reply = xcb_intern_atom_reply(connection, cookie, nullptr);
	if (reply == nullptr) {
		return XCB_ATOM_NONE;
	} else {
		const auto atom = reply->atom;
		free(reply);
		return atom;
	}
}

} // namespace

Window::Window(
	xcb_connection_t *connection,
	xcb_screen_t *screen,
	const char *title,
	uint16_t width,
	uint16_t height
):
	_screenWidth(width),
	_screenHeight(height),
	_windowX((screen->width_in_pixels  - width)  / 2),
	_windowY((screen->height_in_pixels - height) / 2),
	_window(xcb_generate_id(connection)),
	_wmDeleteWindow(getAtom(connection, "WM_DELETE_WINDOW")),
	_inputStates()
{
	// ウィンドウ作成
	const uint32_t mask = XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK;
	const std::array<uint32_t, 2> values{
		screen->black_pixel,
		XCB_EVENT_MASK_EXPOSURE
			| XCB_EVENT_MASK_KEY_PRESS
			| XCB_EVENT_MASK_KEY_RELEASE
			| XCB_EVENT_MASK_STRUCTURE_NOTIFY,
	};
	xcb_create_window(
		connection,
		XCB_COPY_FROM_PARENT,
		_window,
		screen->root,
		_windowX,
		_windowY,
		_screenWidth,
		_screenHeight,
		0,
		XCB_WINDOW_CLASS_INPUT_OUTPUT,
		screen->root_visual,
		mask,
		values.data()
	);

	// タイトル設定
	xcb_change_property(
		connection,
		XCB_PROP_MODE_REPLACE,
		_window,
		XCB_ATOM_WM_NAME,
		XCB_ATOM_STRING,
		8,
		strlen(title),
		title
	);

	// WM_DELETE_WINDOWプロトコルを設定
	//
	// NOTE: 閉じるボタンが押されたことを検知するため。
	xcb_change_property(
		connection,
		XCB_PROP_MODE_REPLACE,
		_window,
		getAtom(connection, "WM_PROTOCOLS"),
		XCB_ATOM_ATOM,
		32,
		1,
		&_wmDeleteWindow
	);

	// ウィンドウを表示
	xcb_map_window(connection, _window);
	xcb_flush(connection);
}

WindowSize Window::getCurrentClientSize(xcb_connection_t *connection) const {
	const auto cookie = xcb_get_geometry(connection, _window);
	const auto reply = xcb_get_geometry_reply(connection, cookie, nullptr);
	if (reply == nullptr) {
		return {0, 0};
	} else {
		const WindowSize size{
			static_cast<uint32_t>(reply->width),
			static_cast<uint32_t>(reply->height),
		};
		free(reply);
		return size;
	}
}

void Window::toggleFullscreen(xcb_connection_t *connection, xcb_screen_t *screen) const {
	const auto wmState = getAtom(connection, "_NET_WM_STATE");
	const auto wmFullscreen = getAtom(connection, "_NET_WM_STATE_FULLSCREEN");

	xcb_client_message_event_t event{};
	event.response_type  = XCB_CLIENT_MESSAGE;
	event.window         = _window;
	event.type           = wmState;
	event.format         = 32;
	event.data.data32[0] = 2; // _NET_WM_STATE_TOGGLE
	event.data.data32[1] = wmFullscreen;

	xcb_send_event(
		connection,
		0,
		screen->root,
		XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT | XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY,
		reinterpret_cast<const char *>(&event)
	);
	xcb_flush(connection);
}
