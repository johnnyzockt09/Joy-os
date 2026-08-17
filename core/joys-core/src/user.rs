//! Benutzer-Informationen.

/// Aktueller Benutzername.
pub fn username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

/// Home-Verzeichnis des aktuellen Benutzers.
pub fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
}

/// Root- oder Standard-Benutzer?
pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        // SAFETY: geteuid hat keine gefährlichen Argumente.
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_is_not_empty() {
        assert!(!username().is_empty());
    }

    #[test]
    fn home_dir_is_absolute() {
        let home = home_dir().expect("home dir");
        assert!(home.is_absolute());
    }
}
