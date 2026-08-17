// gditest.c – Testet die GDI32-Infrastruktur von joys-win (PHASE 10):
// GetDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject,
// SetPixelV, GetPixel, DeleteObject, DeleteDC, ReleaseDC.
//
// Nutzt NUR kernel32 + gdi32 (kein CRT).
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c gditest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB gditest.obj kernel32.lib gdi32.lib /OUT:gditest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console gditest.c -lkernel32 -lgdi32 -o gditest.exe

#include <windows.h>

static int str_len(const char *s) { int n = 0; while (s[n] != 0) n++; return n; }
static void mem_copy(void *d, const void *s, int n) {
    const char *src = (const char *)s; char *dst = (char *)d; int i;
    for (i = 0; i < n; i++) dst[i] = src[i];
}
static char *utoa_into(unsigned long v, char *out) {
    char tmp[32]; int i = 0;
    do { tmp[i++] = (char)('0' + (v % 10)); v /= 10; } while (v != 0);
    while (i > 0) *out++ = tmp[--i];
    return out;
}
static void emit(HANDLE hOut, char *buf, char *end) {
    DWORD written = 0;
    WriteFile(hOut, buf, (DWORD)(end - buf), &written, NULL);
}

void mainCRTStartup(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    char buf[128];
    char *p;
    HDC screen = GetDC(NULL);
    HDC mem = CreateCompatibleDC(screen);
    HBITMAP bmp = CreateCompatibleBitmap(screen, 8, 8);
    HGDIOBJ old = SelectObject(mem, bmp);
    COLORREF c1, c2;

    p = buf;
    mem_copy(p, "dc=", 3); p += 3;
    p = utoa_into((unsigned long)(size_t)(mem != NULL), p);
    mem_copy(p, " bmp=", 5); p += 5;
    p = utoa_into((unsigned long)(size_t)(bmp != NULL), p);
    *p++ = '\n';
    emit(hOut, buf, p);

    c1 = SetPixelV(mem, 1, 1, 0x00FF0000);   /* rot */
    c2 = GetPixel(mem, 1, 1);

    p = buf;
    mem_copy(p, "set=", 4); p += 4;
    p = utoa_into(c1, p);
    mem_copy(p, " get=", 5); p += 5;
    p = utoa_into(c2, p);
    mem_copy(p, " ok=", 4); p += 4;
    p = utoa_into((unsigned long)(c2 == 0x00FF0000), p);
    *p++ = '\n';
    emit(hOut, buf, p);

    SelectObject(mem, old);
    DeleteObject(bmp);
    DeleteDC(mem);
    ReleaseDC(NULL, screen);

    ExitProcess(0);
}
