#pragma once

#include <unordered_map>
#include <xcb/xcb.h>

struct WindowSize {
	uint32_t width;
	uint32_t height;
};

class Window {
private:
	const uint16_t _windowX;
	const uint16_t _windowY;
	const uint16_t _screenWidth;
	const uint16_t _screenHeight;
	const xcb_window_t _window;
	const xcb_atom_t _wmDeleteWindow;
	std::unordered_map<xcb_keycode_t, bool> _inputStates;

public:
	Window(const Window&) = delete;
	Window& operator=(const Window &) = delete;
	Window(Window &&) = default;
	Window& operator=(Window &&) = delete;

	Window(
		xcb_connection_t *connection,
		xcb_screen_t *screen,
		const char *title,
		uint16_t width,
		uint16_t height
	);

	// WARN: このメソッドを呼んだ後はこのインスタンスを利用しないこと。
	void destroy(xcb_connection_t *connection) {
		xcb_destroy_window(connection, _window);
	}

	xcb_window_t getWindow() const {
		return _window;
	}

	bool shouldClose(const xcb_client_message_event_t *event) const {
		return event->data.data32[0] == _wmDeleteWindow;
	}

	void pressKey(xcb_keycode_t code) {
		_inputStates[code] = true;
	}

	void releaseKey(xcb_keycode_t code) {
		_inputStates[code] = false;
	}

	bool getInputState(xcb_keycode_t code) const {
		return _inputStates.count(code) > 0 ? _inputStates.at(code) : false;
	}

	WindowSize getCurrentClientSize(xcb_connection_t *connection) const;

	void toggleFullscreen(xcb_connection_t *connection, xcb_screen_t *screen) const;
};
