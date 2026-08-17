// audiotest.c – Testet die winmm/waveOut-Audio-API von joys-win (PHASE 12).
// Ruft waveOutOpen auf. Ohne Audio-Device liefert es MMSYSERR_NODRIVER (6),
// mit Device 0 — der Return wird ausgegeben (kein Dummy).
//
// Nutzt NUR kernel32 + winmm (kein CRT).
//
// Build (wie hello.c):
//   MSVC:  cl /nologo /GS- /c audiotest.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB audiotest.obj kernel32.lib winmm.lib /OUT:audiotest.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console audiotest.c -lkernel32 -lwinmm -o audiotest.exe

#include <windows.h>
#include <mmsystem.h>

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
static void mem_set(void *d, int c, int n) {
    char *dst = (char *)d; int i;
    for (i = 0; i < n; i++) dst[i] = (char)c;
}
static void emit(HANDLE hOut, char *buf, char *end) {
    DWORD written = 0;
    WriteFile(hOut, buf, (DWORD)(end - buf), &written, NULL);
}

static char sil[640];

int audiotest(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    char buf[128];
    char *p;
    WAVEFORMATEX fmt;
    HWAVEOUT hwo = NULL;
    WAVEHDR hdr;
    MMRESULT rc;

    mem_set(&fmt, 0, sizeof fmt);
    fmt.wFormatTag = WAVE_FORMAT_PCM;
    fmt.nChannels = 2;
    fmt.nSamplesPerSec = 44100;
    fmt.wBitsPerSample = 16;
    fmt.nBlockAlign = 4;
    fmt.nAvgBytesPerSec = 44100 * 4;

    rc = waveOutOpen(&hwo, WAVE_MAPPER, &fmt, 0, 0, 0);
    p = buf;
    mem_copy(p, "waveOutOpen=", 12); p += 12;
    p = utoa_into(rc, p);
    mem_copy(p, " hwo=", 5); p += 5;
    p = utoa_into((unsigned long)(hwo != NULL), p);
    *p++ = '\n';
    emit(hOut, buf, p);

    if (rc == MMSYSERR_NOERROR && hwo != NULL) {
        mem_set(sil, 0, sizeof sil);
        mem_set(&hdr, 0, sizeof hdr);
        hdr.lpData = sil;
        hdr.dwBufferLength = sizeof sil;
        waveOutPrepareHeader(hwo, &hdr, sizeof hdr);
        MMRESULT w = waveOutWrite(hwo, &hdr, sizeof hdr);
        waveOutClose(hwo);
        p = buf;
        mem_copy(p, "write=", 6); p += 6;
        p = utoa_into(w, p);
        mem_copy(p, " done_flag=", 11); p += 11;
        p = utoa_into((unsigned long)(hdr.dwFlags & WHDR_DONE), p);
        *p++ = '\n';
        emit(hOut, buf, p);
    }

    ExitProcess(0);
    return 0;
}

void mainCRTStartup(void) { (void)audiotest(); }
