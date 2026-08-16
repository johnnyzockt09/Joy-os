// hello.c – Minimales Windows-Konsolenprogramm für joys-win (PHASE 6).
//
// Nutzt NUR kernel32.dll (kein CRT): beweist, dass der joys-win-Loader
// Imports auflösen und den Entry-Point ausführen kann.
//
// Build:
//   MSVC:  cl /nologo /GS- /c hello.c && link /nologo /SUBSYSTEM:CONSOLE \
//            /ENTRY:mainCRTStartup /NODEFAULTLIB hello.obj kernel32.lib /OUT:hello.exe
//   MinGW: x86_64-w64-mingw32-gcc -nostdlib -Wl,-e,mainCRTStartup \
//            -Wl,--subsystem,console hello.c -lkernel32 -o hello.exe

#include <windows.h>

void mainCRTStartup(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    const char msg[] = "Hello from Windows!\n";
    DWORD written = 0;
    WriteFile(hOut, msg, (DWORD)(sizeof(msg) - 1), &written, NULL);
    ExitProcess(0);
}
