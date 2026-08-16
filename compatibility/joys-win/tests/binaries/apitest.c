// apitest.c – Testet weitere kernel32-Funktionen von joys-win (PHASE 7).
//
// Nutzt NUR kernel32.dll (kein CRT): Sleep, GetTickCount, GetCurrentProcessId,
// GetCurrentThreadId, GetLastError/SetLastError, VirtualAlloc/VirtualFree,
// GetSystemInfo, lstrlenA, GetCommandLineA.
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c apitest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB apitest.obj kernel32.lib /OUT:apitest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console apitest.c -lkernel32 -o apitest.exe

#include <windows.h>

/* Kleine CRT-freie Helfer. */
static int str_len(const char *s) {
    int n = 0;
    while (s[n] != 0) n++;
    return n;
}

static void mem_copy(void *dst, const void *src, int n) {
    const char *s = (const char *)src;
    char *d = (char *)dst;
    int i;
    for (i = 0; i < n; i++) d[i] = s[i];
}

static char *utoa_into(unsigned long v, char *out) {
    char tmp[32];
    int i = 0;
    do { tmp[i++] = (char)('0' + (v % 10)); v /= 10; } while (v != 0);
    while (i > 0) *out++ = tmp[--i];
    return out;
}

/* Komfort: schreibt label + Zahl in buf, liefert neue Schreibposition. */
static char *w_ul(char *p, const char *label, unsigned long v) {
    int len = str_len(label);
    mem_copy(p, label, len);
    p += len;
    return utoa_into(v, p);
}

static void emit(HANDLE hOut, char *buf, char *end) {
    DWORD written = 0;
    WriteFile(hOut, buf, (DWORD)(end - buf), &written, NULL);
}

void mainCRTStartup(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    char buf[512];
    char *p;

    /* GetLastError/SetLastError */
    SetLastError(0x1234);
    p = buf;
    p = w_ul(p, "GetLastError=", GetLastError());
    *p++ = '\n';
    emit(hOut, buf, p);

    /* GetSystemInfo */
    {
        SYSTEM_INFO si;
        GetSystemInfo(&si);
        p = buf;
        p = w_ul(p, "nproc=", si.dwNumberOfProcessors);
        p = w_ul(p, " page=", si.dwPageSize);
        p = w_ul(p, " gran=", si.dwAllocationGranularity);
        p = w_ul(p, " arch=", si.wProcessorArchitecture);
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    /* GetCurrentProcessId / GetCurrentThreadId / lstrlenA */
    p = buf;
    p = w_ul(p, "pid=", GetCurrentProcessId());
    p = w_ul(p, " tid=", GetCurrentThreadId());
    p = w_ul(p, " lstrlenA=", (unsigned long)lstrlenA("Hallo"));
    *p++ = '\n';
    emit(hOut, buf, p);

    /* Sleep + GetTickCount (Differenz >= 10ms) */
    {
        unsigned long t1 = GetTickCount();
        Sleep(10);
        unsigned long t2 = GetTickCount();
        p = buf;
        p = w_ul(p, "tick1=", t1);
        p = w_ul(p, " tick2=", t2);
        p = w_ul(p, " diff=", t2 - t1);
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    /* VirtualAlloc/VirtualFree */
    {
        LPVOID mem = VirtualAlloc(NULL, 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        char *q = (char *)mem;
        p = buf;
        p = w_ul(p, "valloc=", (unsigned long)(size_t)mem);
        p = w_ul(p, " writable=", q ? (q[0] = 'J', (unsigned long)q[0]) : 0);
        p = w_ul(p, " free=", (unsigned long)VirtualFree(mem, 0, MEM_RELEASE));
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    ExitProcess(0);
}
