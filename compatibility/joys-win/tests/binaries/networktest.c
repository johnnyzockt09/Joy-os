// networktest.c – Testet die ws2_32-Socket-API von joys-win (PHASE 11).
// Erzeugt einen Loopback-Echo: Server bind/listen/accept + Client connect,
// send/recv, Echo zurueck.
//
// Nutzt NUR kernel32 + ws2_32 (kein CRT).
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c networktest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB networktest.obj kernel32.lib ws2_32.lib /OUT:networktest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console networktest.c -lkernel32 -lws2_32 -o networktest.exe

#include <winsock2.h>
#include <windows.h>

static int str_len(const char *s) { int n = 0; while (s[n] != 0) n++; return n; }
static void mem_copy(void *d, const void *s, int n) {
    const char *src = (const char *)s; char *dst = (char *)d; int i;
    for (i = 0; i < n; i++) dst[i] = src[i];
}
static void mem_set(void *d, int c, int n) {
    char *dst = (char *)d; int i;
    for (i = 0; i < n; i++) dst[i] = (char)c;
}
static int mem_cmp(const void *a, const void *b, int n) {
    const char *x = (const char *)a; const char *y = (const char *)b; int i;
    for (i = 0; i < n; i++) if (x[i] != y[i]) return 1;
    return 0;
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
    char buf[256];
    char *p;
    WSADATA wsa;
    SOCKET srv, cli, acc;
    struct sockaddr_in a;
    int alen;
    char ebuf[16];
    int n;
    int ok = 0;

    p = buf;
    mem_copy(p, "wsastartup=", 11); p += 11;
    p = utoa_into((unsigned long)WSAStartup(MAKEWORD(2, 2), &wsa), p);
    *p++ = '\n';
    emit(hOut, buf, p);

    srv = socket(AF_INET, SOCK_STREAM, 0);
    cli = socket(AF_INET, SOCK_STREAM, 0);

    mem_set(&a, 0, sizeof a);
    a.sin_family = AF_INET;
    a.sin_port = 0; /* ephemerer Port */
    a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

    p = buf;
    mem_copy(p, "sock=", 5); p += 5;
    p = utoa_into((unsigned long)(srv != INVALID_SOCKET && cli != INVALID_SOCKET), p);
    mem_copy(p, " bind=", 6); p += 6;
    p = utoa_into((unsigned long)bind(srv, (struct sockaddr *)&a, sizeof a), p);
    *p++ = '\n';
    emit(hOut, buf, p);

    alen = sizeof a;
    if (getsockname(srv, (struct sockaddr *)&a, &alen) == 0) {
        listen(srv, 4);
        if (connect(cli, (struct sockaddr *)&a, sizeof a) == 0) {
            acc = accept(srv, NULL, NULL);
            if (acc != INVALID_SOCKET) {
                n = send(cli, "ping", 4, 0);
                n = recv(acc, ebuf, sizeof ebuf, 0);
                if (n > 0) send(acc, ebuf, n, 0);
                n = recv(cli, ebuf, sizeof ebuf, 0);
                ok = (n == 4 && mem_cmp(ebuf, "ping", 4) == 0);
                closesocket(acc);
            }
        }
    }
    closesocket(cli);
    closesocket(srv);
    WSACleanup();

    p = buf;
    mem_copy(p, "echo=", 5); p += 5;
    if (ok) { mem_copy(p, ebuf, 4); p += 4; }
    else { mem_copy(p, "none", 4); p += 4; }
    mem_copy(p, " net ok=", 8); p += 8;
    p = utoa_into((unsigned long)ok, p);
    *p++ = '\n';
    emit(hOut, buf, p);

    ExitProcess(ok ? 0 : 1);
}
