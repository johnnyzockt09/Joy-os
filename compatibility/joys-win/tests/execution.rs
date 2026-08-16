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
