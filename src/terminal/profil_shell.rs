/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Shell profiles
 * Purpose: Defines supported shells and resolves executable locations
 */

use std::env;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfilShell {
    #[default]
    WindowsPowerShell,
    PowerShell7,
    Cmd,
    GitBash,
}

impl ProfilShell {
    pub const SEMUA: [Self; 4] = [
        Self::WindowsPowerShell,
        Self::PowerShell7,
        Self::Cmd,
        Self::GitBash,
    ];

    pub fn nama(self) -> &'static str {
        match self {
            Self::WindowsPowerShell => "Windows PowerShell",
            Self::PowerShell7 => "PowerShell 7",
            Self::Cmd => "Command Prompt",
            Self::GitBash => "Git Bash",
        }
    }

    fn program_git_bash() -> Option<PathBuf> {
        for variabel in ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(direktori) = env::var_os(variabel) {
                let program = PathBuf::from(direktori).join(r"Git\bin\bash.exe");

                if program.is_absolute() && program.is_file() {
                    return Some(program);
                }
            }
        }

        if let Some(direktori) = env::var_os("LOCALAPPDATA") {
            let program = PathBuf::from(direktori).join(r"Programs\Git\bin\bash.exe");

            if program.is_absolute() && program.is_file() {
                return Some(program);
            }
        }

        let git = Self::cari_program_dalam_path("git.exe")?;

        let direktori = git.parent()?.parent()?;

        let program = direktori.join(r"bin\bash.exe");

        if program.is_file() {
            return Some(program);
        }

        None
    }

    pub fn mendukung_integrasi_shell(self) -> bool {
        matches!(self, Self::WindowsPowerShell | Self::PowerShell7)
    }

    pub fn lokasi_program(self) -> Option<PathBuf> {
        match self {
            Self::WindowsPowerShell => {
                Self::program_sistem(r"System32\WindowsPowerShell\v1.0\powershell.exe")
            }

            Self::PowerShell7 => Self::program_powershell_7(),

            Self::Cmd => Self::program_sistem(r"System32\cmd.exe"),

            Self::GitBash => Self::program_git_bash(),
        }
    }

    pub fn tersedia(self) -> bool {
        self.lokasi_program().is_some()
    }

    pub fn deteksi_tersedia() -> Vec<(Self, PathBuf)> {
        Self::SEMUA
            .into_iter()
            .filter_map(|profil| profil.lokasi_program().map(|lokasi| (profil, lokasi)))
            .collect()
    }

    pub fn default_tersedia() -> Option<Self> {
        Self::SEMUA.into_iter().find(|profil| profil.tersedia())
    }

    pub(super) fn program_sistem(lokasi: &str) -> Option<PathBuf> {
        let direktori = PathBuf::from(env::var_os("SystemRoot")?);
        let program = direktori.join(lokasi);

        (program.is_absolute() && program.is_file()).then_some(program)
    }

    fn program_powershell_7() -> Option<PathBuf> {
        for variabel in ["ProgramW6432", "ProgramFiles"] {
            if let Some(direktori) = env::var_os(variabel) {
                let program = PathBuf::from(direktori).join(r"PowerShell\7\pwsh.exe");

                if program.is_absolute() && program.is_file() {
                    return Some(program);
                }
            }
        }

        Self::cari_program_dalam_path("pwsh.exe")
    }

    fn cari_program_dalam_path(nama: &str) -> Option<PathBuf> {
        let jalur = env::var_os("PATH")?;

        env::split_paths(&jalur)
            .filter(|direktori| direktori.is_absolute())
            .map(|direktori| direktori.join(nama))
            .find(|program| program.is_file())
    }
}
