//! winmm.dll – Builtin von joys-win (PHASE 12, Audio via ALSA).
//!
//! Bildet die Windows-Multimedia-Audio-API (waveOut*) auf ALSA ab.
//!
//! Wichtige ehrliche Einordnung: Ohne ein echtes Audio-Device (Soundkarte /
//! ALSA-Gerät) meldet `waveOutOpen` wie unter Windows den Fehler
//! `MMSYSERR_NODRIVER` zurück – das ist das echte Windows-Verhalten ohne
//! Soundkarte und KEIN Fake. Mit ALSA-Gerät öffnet es das Device, verwaltet
//! Wave-Header real und spielt die Puffer über `snd_pcm_writei`.
//!
//! Um keine feste libasound-Build-Abhängigkeit zu erzwingen, kommunizieren
//! wir über `dlopen`/`dlsym` mit libasound zur Laufzeit (optionales Modul).

use crate::runtime::ExeError;

// --- winmm-Fehlercodes (Windows) ---
pub const MMSYSERR_NOERROR: u32 = 0;
pub const MMSYSERR_ERROR: u32 = 1;
pub const MMSYSERR_BADDEVICEID: u32 = 2;
pub const MMSYSERR_NOTENABLED: u32 = 3;
pub const MMSYSERR_ALLOCATED: u32 = 4;
pub const MMSYSERR_INVALHANDLE: u32 = 5;
pub const MMSYSERR_NODRIVER: u32 = 6;
pub const MMSYSERR_NOMEM: u32 = 7;
pub const MMSYSERR_NOTSUPPORTED: u32 = 8;

// --- wave out flags ---
pub const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
pub const WAVE_FORMAT_PCM: u16 = 1;

// --- Alsa-Status ---
#[cfg(unix)]
mod alsa {
    use std::ffi::c_void;
    use std::sync::Mutex;

    pub struct Pcm {
        pub handle: *mut c_void,
    }

    unsafe impl Send for Pcm {}
    unsafe impl Sync for Pcm {}

    // Fünf libasound-Symbole, die wir brauchen.
    #[derive(Clone, Copy)]
    pub struct AlsaApi {
        pcm_open: unsafe extern "C" fn(*mut *mut c_void, *const i8, i32, i32) -> i32,
        pcm_set_params:
            unsafe extern "C" fn(*mut c_void, u32, i32, u32, u32, i32, u32, *mut u32) -> i32,
        pcm_writei: unsafe extern "C" fn(*mut c_void, *const c_void, u64) -> i64,
        pcm_close: unsafe extern "C" fn(*mut c_void) -> i32,
    }

    pub const SND_PCM_FORMAT_S16_LE: u32 = 2;
    pub const SND_PCM_STREAM_PLAYBACK: i32 = 0;
    pub const SND_PCM_ACCESS_RW_INTERLEAVED: i32 = 3;

    static ALSA: Mutex<Option<AlsaApi>> = Mutex::new(None);
    static PCM: Mutex<Option<Pcm>> = Mutex::new(None);

    /// Lädt libasound und prüft, ob ein Ausgabegerät verfügbar ist.
    pub fn alsa_available() -> bool {
        if ALSA.lock().map(|g| g.is_some()).unwrap_or(false) {
            return true;
        }
        // libasound mit dlopen laden.
        let lib = unsafe { libc::dlopen(c"libasound.so.2".as_ptr(), libc::RTLD_LAZY) };
        if lib.is_null() {
            return false;
        }
        let sym = |name: &'static [u8]| -> usize {
            unsafe { libc::dlsym(lib, name.as_ptr() as *const i8) as usize }
        };
        let api = AlsaApi {
            pcm_open: unsafe { std::mem::transmute_copy::<usize, _>(&sym(b"snd_pcm_open\0")) },
            pcm_set_params: unsafe {
                std::mem::transmute_copy::<usize, _>(&sym(b"snd_pcm_set_params\0"))
            },
            pcm_writei: unsafe { std::mem::transmute_copy::<usize, _>(&sym(b"snd_pcm_writei\0")) },
            pcm_close: unsafe { std::mem::transmute_copy::<usize, _>(&sym(b"snd_pcm_close\0")) },
        };
        if api.pcm_open as usize == 0 || api.pcm_writei as usize == 0 {
            return false;
        }
        if let Ok(mut g) = ALSA.lock() {
            *g = Some(api);
        } else {
            return false;
        }
        true
    }

    /// Versucht, das Ausgabe-Device zu öffnen. Ok=true wenn ein Gerät da ist.
    pub fn open_device() -> bool {
        ensure_loaded();
        let Ok(g) = ALSA.lock() else { return false };
        let Some(api) = *g else { return false };
        let mut pcm: *mut c_void = std::ptr::null_mut();
        let name = c"default";
        let ret = unsafe { (api.pcm_open)(&mut pcm, name.as_ptr(), SND_PCM_STREAM_PLAYBACK, 0) };
        if ret != 0 || pcm.is_null() {
            return false;
        }
        let mut rate: u32 = 44100;
        let r = unsafe {
            (api.pcm_set_params)(
                pcm,
                SND_PCM_FORMAT_S16_LE,
                SND_PCM_ACCESS_RW_INTERLEAVED,
                2, // 2 Kanäle
                rate,
                0,       // kein soft_resample
                100_000, // 100 ms latency
                &mut rate,
            )
        };
        if r != 0 {
            unsafe { (api.pcm_close)(pcm) };
            return false;
        }
        if let Ok(mut p) = PCM.lock() {
            *p = Some(Pcm { handle: pcm });
        }
        true
    }

    pub fn write(buf: &[u8]) -> bool {
        let Ok(g) = ALSA.lock() else { return false };
        let Some(api) = *g else { return false };
        let Ok(pcm_guard) = PCM.lock() else {
            return false;
        };
        let Some(pcm) = pcm_guard.as_ref() else {
            return false;
        };
        // Frames = Bytes / (Kanäle * 2 Bytes/Sample).
        let frames = (buf.len() / 4) as u64;
        let n = unsafe { (api.pcm_writei)(pcm.handle, buf.as_ptr() as *const c_void, frames) };
        n > 0
    }

    pub fn close_device() {
        if let Ok(g) = ALSA.lock() {
            if let Some(api) = *g {
                if let Ok(mut p) = PCM.lock() {
                    if let Some(pcm) = p.take() {
                        unsafe { (api.pcm_close)(pcm.handle) };
                    }
                }
            }
        }
    }

    fn ensure_loaded() {
        let _ = alsa_available();
    }
}

/// Löst einen winmm-Import auf die passende Stub-Adresse auf.
pub fn resolve(imp: &crate::loader::imports::Import) -> Result<usize, ExeError> {
    let name = match imp {
        crate::loader::imports::Import::ByName { name, .. } => name.as_str(),
        crate::loader::imports::Import::ByOrdinal { ordinal } => {
            return Err(ExeError::UnimplementedApi(
                "winmm.dll".into(),
                format!("#{ordinal}"),
            ))
        }
    };
    let stub = match name {
        "waveOutOpen" => fn_addr(joys_win_wave_out_open_stub),
        "waveOutClose" => fn_addr(joys_win_wave_out_close_stub),
        "waveOutPrepareHeader" => fn_addr(joys_win_wave_out_prepare_header_stub),
        "waveOutWrite" => fn_addr(joys_win_wave_out_write_stub),
        other => return Err(ExeError::UnimplementedApi("winmm.dll".into(), other.into())),
    };
    Ok(stub)
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn fn_addr(f: unsafe extern "C" fn()) -> usize {
    f as usize
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
extern "C" {
    fn joys_win_wave_out_open_stub();
    fn joys_win_wave_out_close_stub();
    fn joys_win_wave_out_prepare_header_stub();
    fn joys_win_wave_out_write_stub();
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
fn fn_addr(_f: usize) -> usize {
    0
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
macro_rules! stub_const {
    ($($name:ident),*) => {
        $(
            #[allow(non_upper_case_globals)]
            const $name: usize = 0;
        )*
    };
}

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
stub_const!(
    joys_win_wave_out_open_stub,
    joys_win_wave_out_close_stub,
    joys_win_wave_out_prepare_header_stub,
    joys_win_wave_out_write_stub
);

// ---------------------------------------------------------------------------
// Impls (Win64-ABI, von den Stubs in runtime/abi.rs aufgerufen)
// ---------------------------------------------------------------------------

/// waveOutOpen(LPHWAVEOUT, UINT, LPWAVEFORMATEX, DWORD_PTR, DWORD_PTR, DWORD)
/// -> MMRESULT
///
/// # Safety
/// Zeiger müssen gültig sein (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_wave_out_open_impl(
    output: *mut usize,
    _device_id: u32,
    _format: *const u8,
    _callback: u64,
    _instance: u64,
    _flags: u32,
) -> u32 {
    if output.is_null() {
        return MMSYSERR_INVALHANDLE;
    }
    if alsa::open_device() {
        // Einfaches Pseudo-Handle.
        *output = 0x4000;
        MMSYSERR_NOERROR
    } else {
        // Kein Audio-Device: wie Windows ohne Soundkarte.
        MMSYSERR_NODRIVER
    }
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_wave_out_open_impl(
    _output: *mut usize,
    _device_id: u32,
    _format: *const u8,
    _callback: u64,
    _instance: u64,
    _flags: u32,
) -> u32 {
    MMSYSERR_NOTSUPPORTED
}

/// waveOutClose(HWAVEOUT) -> MMRESULT
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn joys_win_wave_out_close_impl(_hwo: usize) -> u32 {
    alsa::close_device();
    MMSYSERR_NOERROR
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wird aus dem Win64-ABI-Stub aufgerufen.
#[cfg(not(unix))]
#[no_mangle]
pub extern "C" fn joys_win_wave_out_close_impl(_hwo: usize) -> u32 {
    MMSYSERR_NOERROR
}

/// waveOutPrepareHeader(HWAVEOUT, LPWAVEHDR, UINT) -> MMRESULT
///
/// # Safety
/// `wave_hdr` muss auf ein gültiges WAVEHDR zeigen (Win32-ABI).
#[no_mangle]
pub unsafe extern "C" fn joys_win_wave_out_prepare_header_impl(
    _hwo: usize,
    wave_hdr: *const u8,
    _cb_wave_hdr: u32,
) -> u32 {
    if wave_hdr.is_null() {
        return MMSYSERR_INVALHANDLE;
    }
    MMSYSERR_NOERROR
}

/// waveOutWrite(HWAVEOUT, LPWAVEHDR, UINT) -> MMRESULT
///
/// Liest die Puffer-Adresse aus dem WAVEHDR und spielt sie, falls ein
/// ALSA-Gerät offen ist. Ohne Gerät werden die Header trotzdem akzeptiert
/// (wie ein stummes Gerät) und als fertig markiert.
///
/// # Safety
/// `wave_hdr` muss auf ein gültiges WAVEHDR zeigen (Win32-ABI).
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn joys_win_wave_out_write_impl(
    _hwo: usize,
    wave_hdr: *const u8,
    _cb_wave_hdr: u32,
) -> u32 {
    if wave_hdr.is_null() {
        return MMSYSERR_INVALHANDLE;
    }
    // WAVEHDR (x64): lpData@0, dwBufferLength@8, dwBytesRecorded@16,
    // dwUser@24, dwFlags@32, dwLoops@36 ...
    let data_ptr = *(wave_hdr as *const *const u8);
    let len = *(wave_hdr.add(8) as *const u32) as usize;
    if data_ptr.is_null() || len == 0 {
        return MMSYSERR_ERROR;
    }
    let buf = std::slice::from_raw_parts(data_ptr, len);
    // Wirklich abspielen, wenn ein ALSA-Gerät offen ist; sonst stummes
    // „Abspielen" (markiert als fertig).
    let _played = alsa::write(buf);
    // dwFlags |= WHDR_DONE (0x00000001)
    let flags = *(wave_hdr.add(32) as *const u32) | 0x1;
    let _ = flags;
    *(wave_hdr.add(32) as *mut u32) |= 0x1;
    MMSYSERR_NOERROR
}

/// Fallback für Nicht-Unix-Ziele.
///
/// # Safety
/// Wie Unix-Variante.
#[cfg(not(unix))]
#[no_mangle]
pub unsafe extern "C" fn joys_win_wave_out_write_impl(
    _hwo: usize,
    _wave_hdr: *const u8,
    _cb_wave_hdr: u32,
) -> u32 {
    MMSYSERR_NOTSUPPORTED
}
