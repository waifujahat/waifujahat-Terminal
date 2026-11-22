/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / SSH client
 * Purpose : Discovers the native OpenSSH client
 */

use std::path::{Path, PathBuf};

use super::profil_shell::ProfilShell;

mod perintah;
mod profil;

pub use profil::ProfilSsh;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientSsh {
    program: PathBuf,
}

impl ClientSsh {
    pub fn deteksi() -> Option<Self> {
        let program = ProfilShell::program_sistem(r"System32\OpenSSH\ssh.exe")?;

        Some(Self { program })
    }

    pub fn tersedia() -> bool {
        Self::deteksi().is_some()
    }

    pub fn program(&self) -> &Path {
        &self.program
    }
}
