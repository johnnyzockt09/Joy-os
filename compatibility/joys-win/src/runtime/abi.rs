//! Win64-ABI-Brücke (nur x86_64 Linux).
//!
//! Windows-Code ruft unsere Builtins mit der Win64-Konvention auf
//! (RCX, RDX, R8, R9, 5.+ auf dem Stack, Schattenraum 32 Bytes). Die
//! Rust-Impls verwenden das SysV-ABI (RDI, RSI, RDX, RCX, R8, R9).
//! Diese Stubs übersetzen die Register und erhalten die Win64-callee-saved
//! Register RDI/RSI/R12, die SysV-Kalifunktionen zerstören dürfen.
//!
//! Hinweis: Rusts `global_asm!` verwendet auf x86_64 Intel-Syntax.
//!
//! Die per-Makro erzeugten Stubs decken 0–4 Register-Argumente ab.
//! `WriteFile` (5 Argumente inkl. Stack-Argument) und `ExitProcess`
//! (noreturn) werden weiterhin manuell geschrieben.

#![cfg(all(target_arch = "x86_64", target_os = "linux"))]

/// Erzeugt einen Win64→SysV-Stub für 0–4 Argumente.
/// `$mv` sind Intel-Asm-Zeilen, die die Win64-Argumentregister
/// (rcx, rdx, r8, r9) auf die SysV-Register (rdi, rsi, rdx, rcx)
/// abbilden. Für u32-Argumente (DWORD) dabei `mov edi, ecx` verwenden
/// (Zero-Extend), für Zeiger/u64 `mov rdi, rcx`.
macro_rules! stub {
    ($name:ident, $impl:ident, [$($mv:expr),*]) => {
        core::arch::global_asm!(
            concat!(
                ".text\n",
                ".globl ", stringify!($name), "\n",
                ".type ", stringify!($name), ", @function\n",
                stringify!($name), ":\n"
            ),
            "  push rdi",
            "  push rsi",
            "  push r12",
            "  sub rsp, 0x30",
            $($mv,)*
            concat!("  call ", stringify!($impl)),
            "  add rsp, 0x30",
            "  pop r12",
            "  pop rsi",
            "  pop rdi",
            "  ret",
        );
    };
}

// GetStdHandle(DWORD) -> HANDLE
stub!(
    joys_win_get_std_handle_stub,
    joys_win_get_std_handle_impl,
    ["  mov edi, ecx"]
);

// Sleep(DWORD)
stub!(joys_win_sleep_stub, joys_win_sleep_impl, ["  mov edi, ecx"]);

// GetTickCount(void) -> DWORD
stub!(
    joys_win_get_tick_count_stub,
    joys_win_get_tick_count_impl,
    []
);

// GetCurrentProcess(void) -> HANDLE
stub!(
    joys_win_get_current_process_stub,
    joys_win_get_current_process_impl,
    []
);

// GetCurrentProcessId(void) -> DWORD
stub!(
    joys_win_get_current_process_id_stub,
    joys_win_get_current_process_id_impl,
    []
);

// GetCurrentThreadId(void) -> DWORD
stub!(
    joys_win_get_current_thread_id_stub,
    joys_win_get_current_thread_id_impl,
    []
);

// GetLastError(void) -> DWORD
stub!(
    joys_win_get_last_error_stub,
    joys_win_get_last_error_impl,
    []
);

// SetLastError(DWORD)
stub!(
    joys_win_set_last_error_stub,
    joys_win_set_last_error_impl,
    ["  mov edi, ecx"]
);

// VirtualAlloc(LPVOID, SIZE_T, DWORD, DWORD) -> LPVOID
stub!(
    joys_win_virtual_alloc_stub,
    joys_win_virtual_alloc_impl,
    [
        "  mov rdi, rcx",
        "  mov rsi, rdx",
        "  mov edx, r8d",
        "  mov ecx, r9d"
    ]
);

// VirtualFree(LPVOID, SIZE_T, DWORD) -> BOOL
stub!(
    joys_win_virtual_free_stub,
    joys_win_virtual_free_impl,
    ["  mov rdi, rcx", "  mov rsi, rdx", "  mov edx, r8d"]
);

// GetSystemInfo(LPSYSTEM_INFO)
stub!(
    joys_win_get_system_info_stub,
    joys_win_get_system_info_impl,
    ["  mov rdi, rcx"]
);

// lstrlenA(LPCSTR) -> int
stub!(
    joys_win_lstrlen_a_stub,
    joys_win_lstrlen_a_impl,
    ["  mov rdi, rcx"]
);

// GetCommandLineA(void) -> LPSTR
stub!(
    joys_win_get_command_line_a_stub,
    joys_win_get_command_line_a_impl,
    []
);

// ---------------------------------------------------------------------------
// Manuelle Stubs:
// ---------------------------------------------------------------------------
core::arch::global_asm!(
    ".text",
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
    // ReadFile(HANDLE, LPVOID, DWORD, LPDWORD, LPOVERLAPPED) -> BOOL
    ".globl joys_win_read_file_stub",
    ".type joys_win_read_file_stub, @function",
    "joys_win_read_file_stub:",
    "  push r12",
    "  mov r12, [rsp+0x30]",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov edx, r8d",
    "  mov rcx, r9",
    "  mov r8, r12",
    "  call joys_win_read_file_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  ret",
    // CreateFileA(LPCSTR, DWORD, DWORD, LPSECURITY_ATTRIBUTES, DWORD, DWORD, HANDLE)
    // Win64: rcx, rdx, r8d, r9, [rsp+0x28], [rsp+0x30], [rsp+0x38]
    ".globl joys_win_create_file_a_stub",
    ".type joys_win_create_file_a_stub, @function",
    "joys_win_create_file_a_stub:",
    "  push r12",
    "  push r13",
    "  mov r12, [rsp+0x38]",
    "  mov r13, [rsp+0x40]",
    "  sub rsp, 0x38",
    "  mov rdi, rcx",
    "  mov esi, edx",
    "  mov edx, r8d",
    "  mov rcx, r9",
    "  mov r8d, r12d",
    "  mov r9d, r13d",
    "  mov rax, [rsp+0x80]",
    "  mov [rsp+0x08], rax",
    "  call joys_win_create_file_a_impl",
    "  add rsp, 0x38",
    "  pop r13",
    "  pop r12",
    "  ret",
    // GetCurrentDirectoryA(DWORD, LPSTR) -> DWORD
    ".globl joys_win_get_current_directory_a_stub",
    ".type joys_win_get_current_directory_a_stub, @function",
    "joys_win_get_current_directory_a_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov edi, ecx",
    "  mov rsi, rdx",
    "  call joys_win_get_current_directory_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    // SetCurrentDirectoryA(LPCSTR) -> BOOL
    ".globl joys_win_set_current_directory_a_stub",
    ".type joys_win_set_current_directory_a_stub, @function",
    "joys_win_set_current_directory_a_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  call joys_win_set_current_directory_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    // GetFileSize(HANDLE, LPDWORD) -> DWORD
    ".globl joys_win_get_file_size_stub",
    ".type joys_win_get_file_size_stub, @function",
    "joys_win_get_file_size_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  call joys_win_get_file_size_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    // CloseHandle(HANDLE) -> BOOL
    ".globl joys_win_close_handle_stub",
    ".type joys_win_close_handle_stub, @function",
    "joys_win_close_handle_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  call joys_win_close_handle_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
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
    ".size joys_win_write_file_stub, .-joys_win_write_file_stub",
    ".size joys_win_read_file_stub, .-joys_win_read_file_stub",
    ".size joys_win_create_file_a_stub, .-joys_win_create_file_a_stub",
    ".size joys_win_get_current_directory_a_stub, .-joys_win_get_current_directory_a_stub",
    ".size joys_win_set_current_directory_a_stub, .-joys_win_set_current_directory_a_stub",
    ".size joys_win_get_file_size_stub, .-joys_win_get_file_size_stub",
    ".size joys_win_close_handle_stub, .-joys_win_close_handle_stub",
    ".size joys_win_exit_process_stub, .-joys_win_exit_process_stub",
);

core::arch::global_asm!(
    ".text",
    // RegCreateKeyA(HKEY, LPCSTR, PHKEY) -> LONG
    ".globl joys_win_reg_create_key_a_stub",
    ".type joys_win_reg_create_key_a_stub, @function",
    "joys_win_reg_create_key_a_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov rdx, r8",
    "  call joys_win_reg_create_key_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    // RegOpenKeyExA(HKEY, LPCSTR, DWORD, REGSAM, PHKEY) -> LONG
    ".globl joys_win_reg_open_key_ex_a_stub",
    ".type joys_win_reg_open_key_ex_a_stub, @function",
    "joys_win_reg_open_key_ex_a_stub:",
    "  push r12",
    "  mov r12, [rsp+0x30]",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov edx, r8d",
    "  mov ecx, r9d",
    "  mov r8, r12",
    "  call joys_win_reg_open_key_ex_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  ret",
    // RegSetValueExA(HKEY, LPCSTR, DWORD, DWORD, const BYTE*, DWORD) -> LONG
    ".globl joys_win_reg_set_value_ex_a_stub",
    ".type joys_win_reg_set_value_ex_a_stub, @function",
    "joys_win_reg_set_value_ex_a_stub:",
    "  push r12",
    "  mov r12, [rsp+0x38]",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov edx, r8d",
    "  mov ecx, r9d",
    "  mov r8, [rsp+0x60]",
    "  mov r9d, r12d",
    "  call joys_win_reg_set_value_ex_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  ret",
    // RegQueryValueExA(HKEY, LPCSTR, LPDWORD, LPDWORD, LPBYTE, LPDWORD) -> LONG
    ".globl joys_win_reg_query_value_ex_a_stub",
    ".type joys_win_reg_query_value_ex_a_stub, @function",
    "joys_win_reg_query_value_ex_a_stub:",
    "  push r12",
    "  mov r12, [rsp+0x38]",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  mov edx, r8d",
    "  mov rcx, r9",
    "  mov r8, [rsp+0x60]",
    "  mov r9, r12",
    "  call joys_win_reg_query_value_ex_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  ret",
    // RegDeleteKeyA(HKEY, LPCSTR) -> LONG
    ".globl joys_win_reg_delete_key_a_stub",
    ".type joys_win_reg_delete_key_a_stub, @function",
    "joys_win_reg_delete_key_a_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  mov rsi, rdx",
    "  call joys_win_reg_delete_key_a_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    // RegCloseKey(HKEY) -> LONG
    ".globl joys_win_reg_close_key_stub",
    ".type joys_win_reg_close_key_stub, @function",
    "joys_win_reg_close_key_stub:",
    "  push rdi",
    "  push rsi",
    "  push r12",
    "  sub rsp, 0x30",
    "  mov rdi, rcx",
    "  call joys_win_reg_close_key_impl",
    "  add rsp, 0x30",
    "  pop r12",
    "  pop rsi",
    "  pop rdi",
    "  ret",
    ".size joys_win_reg_create_key_a_stub, .-joys_win_reg_create_key_a_stub",
    ".size joys_win_reg_open_key_ex_a_stub, .-joys_win_reg_open_key_ex_a_stub",
    ".size joys_win_reg_set_value_ex_a_stub, .-joys_win_reg_set_value_ex_a_stub",
    ".size joys_win_reg_query_value_ex_a_stub, .-joys_win_reg_query_value_ex_a_stub",
    ".size joys_win_reg_delete_key_a_stub, .-joys_win_reg_delete_key_a_stub",
    ".size joys_win_reg_close_key_stub, .-joys_win_reg_close_key_stub",
);

// ---------------------------------------------------------------------------
// User32 / GDI32 Stubs (Win64 -> SysV)
// ---------------------------------------------------------------------------

/// Generischer Stub für beliebig viele Argumente (bis 4 Register + Stack).
/// `$mv` sind Intel-Asm-Zeilen (Register-Moves + Stack-Moves).
macro_rules! stub_n {
    ($name:ident, $impl:ident, [$($mv:expr),*]) => {
        core::arch::global_asm!(
            concat!(
                ".text\n",
                ".globl ", stringify!($name), "\n",
                ".type ", stringify!($name), ", @function\n",
                stringify!($name), ":\n"
            ),
            "  push r12", "  push r13", "  push r14", "  push r15",
            "  sub rsp, 0x38",
            $($mv,)*
            concat!("  call ", stringify!($impl)),
            "  add rsp, 0x38", "  pop r15", "  pop r14", "  pop r13", "  pop r12",
            "  ret",
        );
    };
}

stub_n!(
    joys_win_register_class_ex_a_stub,
    joys_win_register_class_ex_a_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_create_window_ex_a_stub,
    joys_win_create_window_ex_a_impl,
    [
        "  mov edi, ecx",
        "  mov rsi, rdx",
        "  mov rdx, r8",
        "  mov ecx, r9d",
        "  mov rax, [rsp+0x80]",
        "  mov [rsp+0x08], rax",
        "  mov rax, [rsp+0x88]",
        "  mov [rsp+0x10], rax",
        "  mov rax, [rsp+0x90]",
        "  mov [rsp+0x18], rax",
        "  mov rax, [rsp+0x98]",
        "  mov [rsp+0x20], rax",
        "  mov rax, [rsp+0xA0]",
        "  mov [rsp+0x28], rax",
        "  mov rax, [rsp+0xA8]",
        "  mov [rsp+0x30], rax",
        "  mov rax, [rsp+0xB0]",
        "  mov [rsp+0x38], rax",
        "  mov rax, [rsp+0xB8]",
        "  mov [rsp+0x40], rax"
    ]
);
stub_n!(
    joys_win_show_window_stub,
    joys_win_show_window_impl,
    ["  mov rdi, rcx", "  mov esi, edx"]
);
stub_n!(
    joys_win_update_window_stub,
    joys_win_update_window_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_destroy_window_stub,
    joys_win_destroy_window_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_get_message_a_stub,
    joys_win_get_message_a_impl,
    [
        "  mov rdi, rcx",
        "  mov rsi, rdx",
        "  mov edx, r8d",
        "  mov ecx, r9d"
    ]
);
stub_n!(
    joys_win_translate_message_stub,
    joys_win_translate_message_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_dispatch_message_a_stub,
    joys_win_dispatch_message_a_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_post_message_a_stub,
    joys_win_post_message_a_impl,
    [
        "  mov rdi, rcx",
        "  mov esi, edx",
        "  mov rdx, r8",
        "  mov rcx, r9"
    ]
);
stub_n!(
    joys_win_post_quit_message_stub,
    joys_win_post_quit_message_impl,
    ["  mov edi, ecx"]
);
stub_n!(
    joys_win_def_window_proc_a_stub,
    joys_win_def_window_proc_a_impl,
    [
        "  mov rdi, rcx",
        "  mov esi, edx",
        "  mov rdx, r8",
        "  mov rcx, r9"
    ]
);
stub_n!(
    joys_win_get_dc_stub,
    joys_win_get_dc_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_release_dc_stub,
    joys_win_release_dc_impl,
    ["  mov rdi, rcx", "  mov rsi, rdx"]
);
stub_n!(
    joys_win_get_module_handle_a_stub,
    joys_win_get_module_handle_a_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_create_compatible_dc_stub,
    joys_win_create_compatible_dc_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_create_compatible_bitmap_stub,
    joys_win_create_compatible_bitmap_impl,
    ["  mov rdi, rcx", "  mov esi, edx", "  mov edx, r8d"]
);
stub_n!(
    joys_win_select_object_stub,
    joys_win_select_object_impl,
    ["  mov rdi, rcx", "  mov rsi, rdx"]
);
stub_n!(
    joys_win_delete_object_stub,
    joys_win_delete_object_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_delete_dc_stub,
    joys_win_delete_dc_impl,
    ["  mov rdi, rcx"]
);
stub_n!(
    joys_win_set_pixel_v_stub,
    joys_win_set_pixel_v_impl,
    [
        "  mov rdi, rcx",
        "  mov esi, edx",
        "  mov edx, r8d",
        "  mov ecx, r9d"
    ]
);
stub_n!(
    joys_win_get_pixel_stub,
    joys_win_get_pixel_impl,
    ["  mov rdi, rcx", "  mov esi, edx", "  mov edx, r8d"]
);

// Trampoline: ruft eine Windows-WndProc (Win64-Konvention) aus Rust (SysV).
// SysV-Argumente: rdi=hwnd, rsi=message, rdx=wParam, rcx=lParam, r8=wndproc.
core::arch::global_asm!(
    ".text",
    ".globl joys_win_call_wndproc",
    ".type joys_win_call_wndproc, @function",
    "joys_win_call_wndproc:",
    "  push rbp",
    "  sub rsp, 0x20",
    "  mov r11, r8",
    "  mov r9, rcx",
    "  mov r8, rdx",
    "  mov rdx, rsi",
    "  mov rcx, rdi",
    "  call r11",
    "  add rsp, 0x20",
    "  pop rbp",
    "  ret",
    ".size joys_win_call_wndproc, .-joys_win_call_wndproc",
);
