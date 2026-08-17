// windowtest.c – Testet die User32-Infrastruktur von joys-win (PHASE 9):
// RegisterClassExA, CreateWindowExA, ShowWindow, PostMessageA, GetMessageA,
// DispatchMessageA, PostQuitMessage, DefWindowProcA.
//
// Nutzt NUR kernel32 + user32 (kein CRT).
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c windowtest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB windowtest.obj kernel32.lib user32.lib /OUT:windowtest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console windowtest.c -lkernel32 -luser32 -o windowtest.exe

#include <windows.h>

static int str_len(const char *s) { int n = 0; while (s[n] != 0) n++; return n; }
static void mem_copy(void *d, const void *s, int n) {
    const char *src = (const char *)s; char *dst = (char *)d; int i;
    for (i = 0; i < n; i++) dst[i] = src[i];
}
static void emit(HANDLE hOut, const char *msg) {
    DWORD written = 0;
    WriteFile(hOut, msg, (DWORD)str_len(msg), &written, NULL);
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (msg == WM_CREATE) {
        emit(hOut, "WM_CREATE\n");
    } else if (msg == WM_APP + 1) {
        emit(hOut, "WM_APP+1\n");
        PostQuitMessage(0);
    } else if (msg == WM_DESTROY) {
        emit(hOut, "WM_DESTROY\n");
    }
    return DefWindowProcA(hwnd, msg, wp, lp);
}

void mainCRTStartup(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    WNDCLASSEXA wc;
    HWND hwnd;
    MSG msg;

    wc.cbSize = sizeof(WNDCLASSEXA);
    wc.style = 0;
    wc.lpfnWndProc = WndProc;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hInstance = GetModuleHandleA(NULL);
    wc.hIcon = NULL;
    wc.hCursor = NULL;
    wc.hbrBackground = NULL;
    wc.lpszMenuName = NULL;
    wc.lpszClassName = "JoysWnd";
    wc.hIconSm = NULL;

    if (RegisterClassExA(&wc) == 0) {
        emit(hOut, "register failed\n");
        ExitProcess(1);
    }
    emit(hOut, "register ok\n");

    hwnd = CreateWindowExA(0, "JoysWnd", "Joys Testfenster", WS_OVERLAPPEDWINDOW,
                           10, 10, 300, 200, NULL, NULL, wc.hInstance, NULL);
    if (hwnd == NULL) {
        emit(hOut, "create failed\n");
        ExitProcess(1);
    }
    emit(hOut, "create ok\n");

    ShowWindow(hwnd, SW_SHOW);
    PostMessageA(hwnd, WM_APP + 1, 0, 0);

    while (GetMessageA(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    emit(hOut, "loop end\n");
    ExitProcess(0);
}
