use std::io;
use std::time::Duration;

use x11_clipboard::{Atom, Clipboard as X11Clipboard};

/// A self-contained clipboard backend that talks directly to X11.
/// Sets both PRIMARY and CLIPBOARD selections.
pub struct Clipboard {
    inner: X11Clipboard,
}

impl Clipboard {
    /// Create a new clipboard handle.
    pub fn new() -> io::Result<Self> {
        let inner = X11Clipboard::new().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("failed to connect to X11 clipboard: {e}"),
            )
        })?;
        Ok(Self { inner })
    }

    /// Read text from the CLIPBOARD selection.
    pub fn get_text(&self) -> io::Result<String> {
        let atoms = &self.inner.getter.atoms;
        let val = self
            .inner
            .load(
                atoms.clipboard,
                atoms.utf8_string,
                atoms.property,
                Duration::from_secs(3),
            )
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to read clipboard: {e}"),
                )
            })?;

        String::from_utf8(val).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("clipboard content is not valid UTF-8: {e}"),
            )
        })
    }

    /// Clear both clipboard selections by setting them to empty, then exit.
    pub fn clear(&self) -> io::Result<()> {
        self.store(self.inner.setter.atoms.clipboard, "")?;
        self.store(self.inner.setter.atoms.primary, "")?;
        Ok(())
    }

    /// Internal: store bytes into a specific selection atom.
    fn store(&self, selection: Atom, text: &str) -> io::Result<()> {
        let atoms = &self.inner.setter.atoms;
        self.inner
            .store(selection, atoms.utf8_string, text.as_bytes())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to set clipboard: {e}"),
                )
            })
    }
}

/// Fork a background daemon that sets the clipboard and stays alive
/// to serve paste requests until another application takes ownership.
///
/// The parent process returns immediately; the forked child creates
/// its own X11 connection, stores the text, and blocks.
pub fn set_persistent(text: &str) -> io::Result<()> {
    // Fork first — child will create its own X11 connection.
    match unsafe { libc::fork() } {
        -1 => Err(io::Error::last_os_error()),
        0 => {
            // Child process — detach from terminal
            unsafe {
                libc::setsid();
            }

            // Create a fresh clipboard connection in the child
            let cb = Clipboard::new().unwrap_or_else(|e| {
                eprintln!("copy daemon: {e}");
                std::process::exit(1);
            });

            // Store in both selections
            if let Err(e) = cb.store(cb.inner.setter.atoms.clipboard, text) {
                eprintln!("copy daemon: {e}");
                std::process::exit(1);
            }
            let _ = cb.store(cb.inner.setter.atoms.primary, text);

            // Block until we lose selection ownership (another app copies).
            let atoms = &cb.inner.getter.atoms;
            let _ = cb.inner.load_wait(atoms.clipboard, atoms.utf8_string, atoms.property);

            std::process::exit(0);
        }
        _parent_pid => {
            // Parent — return immediately
            Ok(())
        }
    }
}
