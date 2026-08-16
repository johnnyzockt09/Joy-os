//! Win64-ABI-Brücke (nur x86_64 Linux).
//!
//! Windows-Code ruft unsere Builtins mit der Win64-Konvention auf
//! (RCX, RDX, R8, R9, 5.+ auf dem Stack, Schattenraum 32 Bytes). Die
//! Rust-Impls verwenden das SysV-ABI (RDI, RSI, RDX, RCX, R8, R9).
//! Diese Stubs übersetzen die Register und erhalten die Win64-callee-saved
//! Register RDI/RSI, die SysV-Kalifunktionen zerstören dürfen.
//!
//! Hinweis: Rusts `global_asm!` verwendet auf x86_64 Intel-Syntax.

#![cfg(all(target_arch = "x86_64", target_os = "linux"))]

core::arch::global_asm!(
    ".text",
    // GetStdHandle(u32) -> i64   (Win64: rcx = nStdHandle)
    ".globl joys_win_get_std_handle_stub",
    ".type joys_win_get_std_handle_stub, @function",
    "joys_win_get_std_handle_stub:",
    "  push rdi",
    "  sub rsp, 0x30",
    "  mov edi, ecx",
    "  call joys_win_get_std_handle_impl",
    "  add rsp, 0x30",
    "  pop rdi",
    "  ret",
    // WriteFile(HANDLE, LPCVOID, DWORD, LPDWORD, LPOVERLAPPED) -> BOOL
    // Win64: rcx, rdx, r8d, r9, [rsp+0x28]
    ".globl joys_win_write_file_stub",
    ".type joys_win_write_file_stub, @function",
    "joys_win_write_file_stub:",
    "  push r12",
    "  mov r12, [rsp+0x30]",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov edx, r8d",
    "  mov rcx, r9",
    "  mov r8, r12",
    "  call joys_win_write_file_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  ret",
    // ExitProcess(UINT) -> !   (Win64: rcx = uExitCode)
    ".globl joys_win_exit_process_stub",
    ".type joys_win_exit_process_stub, @function",
    "joys_win_exit_process_stub:",
    "  push rdi",
    "  sub rsp, 0x30",
    "  mov edi, ecx",
    "  call joys_win_exit_process_impl",
    "  ud2",
    ".size joys_win_get_std_handle_stub, .-joys_win_get_std_handle_stub",
    ".size joys_win_write_file_stub, .-joys_win_write_file_stub",
    ".size joys_win_exit_process_stub, .-joys_win_exit_process_stub",
);
