#include <cstdint>
#include <stdexcept>
#include <string>
#include <Windows.h>

namespace {

constexpr LPCWSTR WINDOW_CLASS_NAME = L"SkdWindow\0";

LRESULT CALLBACK windowProc(HWND window, UINT msg, WPARAM wParam, LPARAM lParam) {
	switch (msg) {
		case WM_DESTROY:
			PostQuitMessage(0);
			return 0;
		default:
			return DefWindowProcW(window, msg, wParam, lParam);
	}
}

void RegisterWindowClass(HINSTANCE inst) {
	WNDCLASSEXW windowClass;
	windowClass.cbSize = sizeof(WNDCLASSEXW);

	if (GetClassInfoExW(inst, WINDOW_CLASS_NAME, &windowClass)) {
		return;
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
		throw std::runtime_error("failed to register a window class.");
	}
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
extern "C" void *get_instance() {
	return GetModuleHandleW(nullptr);
}

/// ウィンドウを作成する関数
///
/// 成功時はウィンドウハンドルを返し、
/// 失敗時はエラーダイアログを表示してnullptrを返す。
extern "C" void *create_window(const wchar_t *title, uint32_t width, uint32_t height) {
	try {
		constexpr DWORD style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
		const auto inst = GetModuleHandleW(nullptr);
		RegisterWindowClass(inst);
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
			throw std::runtime_error("failed to create a window.");
		}

		ShowWindow(window, SW_SHOWDEFAULT);
		UpdateWindow(window);
		return window;
	} catch (const std::exception &e) {
		const std::string msg(e.what());
		const std::wstring wmsg(msg.cbegin(), msg.cend());
		MessageBoxW(nullptr, wmsg.c_str(), L"Error", MB_OK | MB_ICONERROR);
		return nullptr;
	}
}

/// ウィンドウを閉じる関数
extern "C" void destroy_window(void *window) {
	if (window != nullptr) {
		DestroyWindow(static_cast<HWND>(window));
	}
}

/// 溜まったウィンドウイベントを処理する関数
///
/// ウィンドウが閉じられたならば0、そうでないなら0以外を返す。
extern "C" uint8_t process_window_events() {
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

struct WindowSize {
	uint32_t width;
	uint32_t height;
};

/// 現在のウィンドウクライアントサイズを取得する関数
extern "C" WindowSize get_current_client_size(void *window) {
	RECT rect;
	GetClientRect(static_cast<HWND>(window), &rect);
	return {
		static_cast<uint32_t>(rect.right  - rect.left),
		static_cast<uint32_t>(rect.bottom - rect.top),
	};
}
