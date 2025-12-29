//! Windowsにおけるウィンドウライブラリ
//!
//! NOTE: Rustにはお手軽なウィンドウライブラリがないため、仕方なくC++で書いている。
//!
//! WARN: マルチスレッド対応をしていないため、必ず単一スレッドで動作すること。

#include <cstdint>
#include <Windows.h>

namespace {

constexpr LPCWSTR WINDOW_CLASS_NAME = L"SkdWindow\0";

HWND g_window = nullptr;

LRESULT CALLBACK windowProc(HWND window, UINT msg, WPARAM wParam, LPARAM lParam) {
	switch (msg) {
		case WM_DESTROY:
			PostQuitMessage(0);
			return 0;
		default:
			return DefWindowProcW(window, msg, wParam, lParam);
	}
}

bool RegisterWindowClass(HINSTANCE inst) {
	WNDCLASSEXW windowClass;
	windowClass.cbSize = sizeof(WNDCLASSEXW);

	if (GetClassInfoExW(inst, WINDOW_CLASS_NAME, &windowClass)) {
		return true;
	}

	windowClass.style         = CS_CLASSDC;
	windowClass.lpfnWndProc   = windowProc;
	windowClass.cbClsExtra    = 0;
	windowClass.cbWndExtra    = 0;
	windowClass.hInstance     = inst;
	windowClass.hIcon         = nullptr;
	windowClass.hCursor       = nullptr;
	windowClass.hbrBackground = nullptr;
	windowClass.lpszMenuName  = nullptr;
	windowClass.lpszClassName = WINDOW_CLASS_NAME;
	windowClass.hIconSm       = nullptr;
	if (RegisterClassExW(&windowClass) == 0) {
		return false;
	}
	return true;
}

void adjustWindowSize(DWORD style, int sceneWidth, int sceneHeight, int &windowWidth, int &windowHeight) {
	RECT rect = { 0, 0, sceneWidth, sceneHeight };
	AdjustWindowRect(&rect, style, 0);
	windowWidth  = rect.right  - rect.left;
	windowHeight = rect.bottom - rect.top;
}

void calculateWindowPlacement(int w, int h, int &x, int &y) {
	const auto frgndWindow = GetForegroundWindow();
	const auto monitor = frgndWindow != nullptr
		? MonitorFromWindow(frgndWindow, MONITOR_DEFAULTTONEAREST)
		: MonitorFromPoint({0, 0}, MONITOR_DEFAULTTOPRIMARY);
	MONITORINFO mi;
	mi.cbSize = sizeof(mi);
	if (monitor != nullptr && GetMonitorInfoW(monitor, &mi)) {
		int width  = mi.rcWork.right  - mi.rcWork.left;
		int height = mi.rcWork.bottom - mi.rcWork.top;
		x = mi.rcWork.left + (width  - w) / 2;
		y = mi.rcWork.top  + (height - h) / 2;
	} else {
		x = 0;
		y = 0;
	}
}

} // namespace

/// インスタンスハンドルを取得する関数
extern "C" void *get_instance_handle() {
	return GetModuleHandleW(nullptr);
}

/// ウィンドウハンドルを取得する関数
///
/// ウィンドウが存在しない場合、nullptrを返す。
extern "C" void *get_window_handle() {
	return g_window;
}

/// ウィンドウを作成する関数
///
/// 失敗時や既に作成済みの場合は0を返す。
extern "C" uint8_t create_window(const wchar_t *title, uint32_t width, uint32_t height) {
	constexpr DWORD style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
	const auto inst = GetModuleHandleW(nullptr);
	if (!RegisterWindowClass(inst)) {
		return 0;
	}
	int w, h;
	adjustWindowSize(style, static_cast<int>(width), static_cast<int>(height), w, h);
	int x, y;
	calculateWindowPlacement(w, h, x, y);

	const auto window = CreateWindowExW(
		0,
		WINDOW_CLASS_NAME,
		title,
		style,
		x,
		y,
		w,
		h,
		NULL,
		NULL,
		inst,
		NULL
	);
	if (window == nullptr) {
		return 0;
	}

	ShowWindow(window, SW_SHOWDEFAULT);
	UpdateWindow(window);

	g_window = window;
	return 1;
}

/// ウィンドウを閉じる関数
extern "C" void destroy_window() {
	if (g_window != nullptr) {
		DestroyWindow(g_window);
		g_window = nullptr;
	}
}

struct WindowSize {
	uint32_t width;
	uint32_t height;
};

/// 溜まったウィンドウイベントを処理する関数
///
/// ウィンドウが存在しない場合や・ウィンドウが閉じられた場合は0を返す。
extern "C" uint8_t process_window_events() {
	if (g_window == nullptr) {
		return 0;
	}

	MSG msg;
	while (true) {
		if (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE) == 0) {
			return 1;
		}
		if (msg.message == WM_QUIT) {
			return 0;
		}
		TranslateMessage(&msg);
		DispatchMessageW(&msg);
	}
}

/// 現在のウィンドウクライアントサイズを取得する関数
///
/// ウィンドウが存在しない場合、0x0のサイズを返す。
extern "C" WindowSize get_current_client_size() {
	if (g_window == nullptr) {
		return {0, 0};
	}

	RECT rect;
	if (!GetClientRect(g_window, &rect)) {
		return {0, 0};
	}

	return {
		static_cast<uint32_t>(rect.right  - rect.left),
		static_cast<uint32_t>(rect.bottom - rect.top),
	};
}
