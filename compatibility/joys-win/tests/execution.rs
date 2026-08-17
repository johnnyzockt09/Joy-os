//! End-to-End-Tests für die Ausführung von Windows-Programmen (PHASE 6).
//!
//! `hello.exe` ist ein mit MSVC erzeugtes, minimiertes PE32+ (nur kernel32:
//! GetStdHandle, WriteFile, ExitProcess, kein CRT). Es wird als Fixture
//! mitgeliefert; Regenerierung: `scripts/build-hello.sh`.
//!
//! Die eigentliche Ausführung (runtime::run) ist nur auf x86_64-Linux
//! unterstützt, daher sind diese Tests entsprechend eingeschränkt.

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::process::Command;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn hello_exe_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/binaries/hello.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn apitest_exe_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/binaries/apitest.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn filetest_exe_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/binaries/filetest.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn windowtest_exe_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/binaries/windowtest.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn gditest_exe_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/binaries/gditest.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn networktest_exe_path() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/binaries/networktest.exe"
    )
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn analyzes_hello_exe() {
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .arg(hello_exe_path())
        .output()
        .expect("joys-win starten");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("EXE"), "Analyse-Output: {stdout}");
    assert!(stdout.contains("WriteFile"), "Imports fehlen: {stdout}");
    assert!(stdout.contains("GetStdHandle"), "Imports fehlen: {stdout}");
    assert!(stdout.contains("ExitProcess"), "Imports fehlen: {stdout}");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_hello_exe() {
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", hello_exe_path()])
        .output()
        .expect("joys-win run starten");
    // ExitProcess(0) -> Prozess verlässt mit 0.
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Hello from Windows!"),
        "Erwartet 'Hello from Windows!', bekam: {stdout}"
    );
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn missing_entry_point_is_error() {
    // `analyze`-Modus mit einer Datei, die kein valides PE ist.
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", "/etc/hostname"])
        .output()
        .expect("joys-win run starten");
    assert_eq!(out.status.code(), Some(3), "kein PE -> Exit-Code 3");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_apitest_exe() {
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", apitest_exe_path()])
        .output()
        .expect("joys-win run starten");
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("GetLastError=4660"), "stdout: {stdout}");
    assert!(stdout.contains("nproc="), "GetSystemInfo fehlt: {stdout}");
    assert!(stdout.contains("page=4096"), "Seitengröße fehlt: {stdout}");
    assert!(stdout.contains("arch=9"), "AMD64 fehlt: {stdout}");
    assert!(stdout.contains("lstrlenA=5"), "lstrlenA fehlt: {stdout}");
    assert!(
        stdout.contains(" diff="),
        "Sleep/GetTickCount fehlt: {stdout}"
    );
    assert!(
        stdout.contains("valloc=") && stdout.contains("free=1"),
        "VirtualAlloc/VirtualFree fehlt: {stdout}"
    );
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_filetest_exe() {
    // In einem Temp-Verzeichnis ausführen, damit die Datei sauber entsteht.
    let dir = std::env::temp_dir().join(format!("joys-filetest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .current_dir(&dir)
        .env("HOME", &dir)
        .args(["run", filetest_exe_path()])
        .output()
        .expect("joys-win run starten");
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("cwd="), "cwd fehlt: {stdout}");
    assert!(
        stdout.contains("write_ok=1 written=12 size=12"),
        "Datei schreiben fehlt: {stdout}"
    );
    assert!(
        stdout.contains("content=Hello file!"),
        "Datei lesen fehlt: {stdout}"
    );
    assert!(
        stdout.contains("reg_create=0 set=0 get=0 value=registry works"),
        "Registry fehlt: {stdout}"
    );
    // Tatsächlich geschriebene Datei + Registry-Datei prüfen.
    let file_content = std::fs::read_to_string(dir.join("joys_test.txt")).unwrap_or_default();
    assert_eq!(file_content, "Hello file!\n");
    let reg_file = dir.join(".joys/windows/registry/HKCU/Software/Joys/FileTest/Greeting@1");
    assert!(
        reg_file.exists(),
        "Registry-Datei fehlt: {}",
        reg_file.display()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_windowtest_exe() {
    // User32: Fensterklasse, Fenster, Message-Loop (PHASE 9).
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", windowtest_exe_path()])
        .output()
        .expect("joys-win run starten");
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("register ok"), "RegisterClassExA: {stdout}");
    assert!(stdout.contains("WM_CREATE"), "WM_CREATE fehlt: {stdout}");
    assert!(stdout.contains("create ok"), "CreateWindowExA: {stdout}");
    assert!(
        stdout.contains("WM_APP+1"),
        "PostMessage/Dispatch: {stdout}"
    );
    assert!(stdout.contains("loop end"), "Message-Loop: {stdout}");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_gditest_exe() {
    // GDI32: Memory-DC, Bitmap, Pixel-Roundtrip (PHASE 10).
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", gditest_exe_path()])
        .output()
        .expect("joys-win run starten");
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("dc=1 bmp=1"), "DC/Bitmap: {stdout}");
    assert!(
        stdout.contains("get=16711680 ok=1"),
        "Pixel-Roundtrip rot (0xFF0000): {stdout}"
    );
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn runs_networktest_exe() {
    // ws2_32: Loopback-Echo über echte Sockets (PHASE 11).
    let out = Command::new(env!("CARGO_BIN_EXE_joys-win"))
        .args(["run", networktest_exe_path()])
        .output()
        .expect("joys-win run starten");
    assert!(
        out.status.success(),
        "status: {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("wsastartup=0"), "WSAStartup: {stdout}");
    assert!(stdout.contains("bind=0"), "bind: {stdout}");
    assert!(
        stdout.contains("echo=ping net ok=1"),
        "Loopback-Echo: {stdout}"
    );
}
