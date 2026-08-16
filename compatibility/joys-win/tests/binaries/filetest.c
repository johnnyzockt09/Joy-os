// filetest.c – Testet die Datei- und Registry-APIs von joys-win (PHASE 8).
//
// Nutzt NUR kernel32.dll + advapi32.dll (kein CRT):
//   CreateFileA/WriteFile/ReadFile/CloseHandle/GetCurrentDirectoryA/GetFileSize
//   RegCreateKeyA/RegSetValueExA/RegQueryValueExA/RegCloseKey
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c filetest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB filetest.obj kernel32.lib advapi32.lib /OUT:filetest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console filetest.c -lkernel32 -ladvapi32 -o filetest.exe

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
static char *w_str(char *p, const char *s) {
    int len = str_len(s); mem_copy(p, s, len); return p + len;
}
static char *w_ul(char *p, const char *label, unsigned long v) {
    return utoa_into(v, w_str(p, label));
}
static void emit(HANDLE hOut, char *buf, char *end) {
    DWORD written = 0;
    WriteFile(hOut, buf, (DWORD)(end - buf), &written, NULL);
}

void mainCRTStartup(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    char buf[512];
    char *p;

    /* Aktuelles Verzeichnis */
    {
        char cwd[256];
        DWORD n = GetCurrentDirectoryA(sizeof(cwd), cwd);
        p = buf;
        p = w_str(p, "cwd=");
        mem_copy(p, cwd, (int)n); p += n;
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    /* Datei schreiben */
    {
        HANDLE f = CreateFileA("joys_test.txt", GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                               FILE_ATTRIBUTE_NORMAL, NULL);
        DWORD written = 0;
        const char msg[] = "Hello file!\n";
        BOOL ok = (f != INVALID_HANDLE_VALUE) &&
                  WriteFile(f, msg, (DWORD)(sizeof(msg) - 1), &written, NULL);
        DWORD size = (f != INVALID_HANDLE_VALUE) ? GetFileSize(f, NULL) : 0;
        if (f != INVALID_HANDLE_VALUE) CloseHandle(f);
        p = buf;
        p = w_str(p, "write_ok="); p = w_ul(p, "", ok);
        p = w_str(p, " written="); p = w_ul(p, "", written);
        p = w_str(p, " size="); p = w_ul(p, "", size);
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    /* Datei lesen */
    {
        HANDLE f = CreateFileA("joys_test.txt", GENERIC_READ, 0, NULL, OPEN_EXISTING,
                               FILE_ATTRIBUTE_NORMAL, NULL);
        char rd[64] = {0};
        DWORD got = 0;
        BOOL ok = (f != INVALID_HANDLE_VALUE) &&
                  ReadFile(f, rd, sizeof(rd) - 1, &got, NULL);
        if (f != INVALID_HANDLE_VALUE) CloseHandle(f);
        p = buf;
        p = w_str(p, "read_ok="); p = w_ul(p, "", ok);
        p = w_str(p, " got="); p = w_ul(p, "", got);
        p = w_str(p, " content=");
        mem_copy(p, rd, (int)got); p += got;
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    /* Registry */
    {
        HKEY hk = NULL;
        LONG rc = RegCreateKeyA(HKEY_CURRENT_USER, "Software\\Joys\\FileTest", &hk);
        DWORD type = REG_SZ;
        const char val[] = "registry works";
        DWORD valLen = (DWORD)(sizeof(val)); /* inkl. NUL */
        LONG rcSet = ERROR_SUCCESS;
        LONG rcGet = ERROR_SUCCESS;
        char out[64] = {0};
        DWORD outLen = sizeof(out);
        if (rc == ERROR_SUCCESS && hk) {
            rcSet = RegSetValueExA(hk, "Greeting", 0, type, (const BYTE *)val, valLen);
            rcGet = RegQueryValueExA(hk, "Greeting", NULL, &type, (BYTE *)out, &outLen);
            RegCloseKey(hk);
        }
        p = buf;
        p = w_str(p, "reg_create="); p = w_ul(p, "", (unsigned long)rc);
        p = w_str(p, " set="); p = w_ul(p, "", (unsigned long)rcSet);
        p = w_str(p, " get="); p = w_ul(p, "", (unsigned long)rcGet);
        p = w_str(p, " value=");
        mem_copy(p, out, (int)outLen); p += (int)outLen;
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    ExitProcess(0);
}
