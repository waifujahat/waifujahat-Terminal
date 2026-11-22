/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PTY session
 * Purpose: Owns the PowerShell process, PTY, resizing, and worker lifetime.
 */

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

mod io;

use super::{
    profil_shell::ProfilShell,
    proses_shell::{ProsesTerminal, cari_shell_turunan, potret_proses},
    tombol::TombolTerminal,
};

use io::{jalankan_pembaca, jalankan_penulis};

use anyhow::{Context, Result, anyhow};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

#[derive(Debug)]
pub enum PesanTerminal {
    Keluaran(Vec<u8>),
    Kesalahan(String),
    Berakhir,
}

pub struct SesiTerminal {
    profil: ProfilShell,
    direktori_awal: Option<PathBuf>,
    master: Box<dyn MasterPty + Send>,
    proses: Box<dyn Child + Send + Sync>,
    pengirim_masukan: Sender<Vec<u8>>,
    penerima_peristiwa: Receiver<PesanTerminal>,
}

impl SesiTerminal {
    pub fn baru() -> Result<Self> {
        let profil = ProfilShell::default_tersedia()
            .ok_or_else(|| anyhow!("Tidak ada shell yang tersedia"))?;

        Self::baru_dengan_profil(profil)
    }

    pub fn baru_dengan_profil(profil: ProfilShell) -> Result<Self> {
        Self::baru_dengan_direktori(profil, None)
    }

    pub fn baru_dengan_direktori(profil: ProfilShell, direktori: Option<&Path>) -> Result<Self> {
        let program = profil
            .lokasi_program()
            .ok_or_else(|| anyhow!("Shell {} tidak tersedia", profil.nama()))?;

        let sistem_pty = native_pty_system();

        let pasangan = sistem_pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal membuat pseudoterminal")?;

        let mut perintah = CommandBuilder::new(program);

        let direktori_awal = direktori
            .filter(|jalur| Self::direktori_awal_valid(jalur))
            .map(Path::to_path_buf);

        if let Some(jalur) = direktori_awal.as_deref() {
            perintah.cwd(jalur);

            if profil == ProfilShell::GitBash {
                perintah.env("CHERE_INVOKING", "1");
            }
        }

        match profil {
            ProfilShell::WindowsPowerShell | ProfilShell::PowerShell7 => {
                perintah.arg("-NoLogo");
                perintah.arg("-NoProfile");
                perintah.arg("-NoExit");
                perintah.arg("-Command");
                perintah.arg(include_str!("integrasi_powershell.ps1"));
            }

            ProfilShell::Cmd => {
                perintah.arg("/D");
            }

            ProfilShell::GitBash => {
                perintah.arg("--login");
                perintah.arg("-i");
            }
        }

        let proses = pasangan
            .slave
            .spawn_command(perintah)
            .with_context(|| format!("Gagal menjalankan {}", profil.nama()))?;

        drop(pasangan.slave);

        let pembaca = pasangan
            .master
            .try_clone_reader()
            .context("Gagal membuka pembaca terminal")?;

        let penulis = pasangan
            .master
            .take_writer()
            .context("Gagal membuka penulis terminal")?;

        let (pengirim_masukan, penerima_masukan) = mpsc::channel();
        let (pengirim_peristiwa, penerima_peristiwa) = mpsc::channel();

        let pengirim_kesalahan = pengirim_peristiwa.clone();

        thread::Builder::new()
            .name("waifujahat-pty-writer".into())
            .spawn(move || {
                jalankan_penulis(penulis, penerima_masukan, pengirim_kesalahan);
            })
            .context("Gagal membuat thread penulis")?;

        thread::Builder::new()
            .name("waifujahat-pty-reader".into())
            .spawn(move || {
                jalankan_pembaca(pembaca, pengirim_peristiwa);
            })
            .context("Gagal membuat thread pembaca")?;

        Ok(Self {
            profil,
            direktori_awal,
            master: pasangan.master,
            proses,
            pengirim_masukan,
            penerima_peristiwa,
        })
    }

    fn direktori_awal_valid(direktori: &Path) -> bool {
        if !direktori.is_absolute() {
            return false;
        }

        #[cfg(windows)]
        {
            use std::path::{Component, Prefix};

            let drive_valid = matches!(
                direktori.components().next(),
                Some(Component::Prefix(awalan))
                    if matches!(awalan.kind(), Prefix::Disk(_))
            );

            if !drive_valid {
                return false;
            }
        }

        direktori.is_dir()
    }

    pub fn direktori_awal(&self) -> Option<&Path> {
        self.direktori_awal.as_deref()
    }

    pub fn profil(&self) -> ProfilShell {
        self.profil
    }
    pub fn id_proses(&self) -> Option<u32> {
        self.proses.process_id()
    }

    pub fn deteksi_shell_turunan(&self) -> Result<Vec<ProsesTerminal>> {
        let Some(akar) = self.id_proses() else {
            return Ok(Vec::new());
        };

        let daftar = potret_proses()?;

        Ok(cari_shell_turunan(akar, &daftar))
    }
    pub fn kirim(&self, masukan: &str) -> Result<()> {
        self.kirim_bytes(masukan.as_bytes())
    }

    pub fn kirim_bytes(&self, masukan: &[u8]) -> Result<()> {
        self.pengirim_masukan
            .send(masukan.to_vec())
            .map_err(|_| anyhow!("Koneksi input terminal terputus"))
    }

    pub fn kirim_tombol(&self, tombol: TombolTerminal<'_>) -> Result<()> {
        if let Some(bytes) = tombol.ke_bytes() {
            self.kirim_bytes(&bytes)?;
        }

        Ok(())
    }

    pub fn ubah_ukuran(&self, kolom: u16, baris: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows: baris.max(1),
                cols: kolom.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal mengubah ukuran ConPTY")
    }

    pub fn try_ambil(&self) -> Option<PesanTerminal> {
        self.penerima_peristiwa.try_recv().ok()
    }
}

impl Drop for SesiTerminal {
    fn drop(&mut self) {
        let _ = self.proses.kill();
    }
}
