/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Settings storage
 * Purpose : Loads and persists terminal settings safely
 */

use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use tempfile::NamedTempFile;

use super::PengaturanTerminal;

const NAMA_DIREKTORI: &str = "Waifujahat Terminal";
const NAMA_BERKAS: &str = "settings.toml";

#[derive(Clone, Debug)]
pub struct PenyimpananPengaturan {
    jalur: PathBuf,
}

impl PenyimpananPengaturan {
    pub fn baru(jalur: impl Into<PathBuf>) -> Self {
        Self {
            jalur: jalur.into(),
        }
    }

    pub fn bawaan() -> Result<Self> {
        let direktori = env::var_os("APPDATA")
            .map(PathBuf::from)
            .context("Variabel lingkungan APPDATA tidak tersedia")?;

        ensure!(direktori.is_absolute(), "Direktori APPDATA tidak valid");

        Ok(Self::baru(direktori.join(NAMA_DIREKTORI).join(NAMA_BERKAS)))
    }

    pub fn jalur(&self) -> &Path {
        &self.jalur
    }

    fn uraikan(&self, isi: &str) -> Result<PengaturanTerminal> {
        let isi = isi.strip_prefix('\u{feff}').unwrap_or(isi);

        let pengaturan: PengaturanTerminal = toml::from_str(isi)
            .with_context(|| format!("Konfigurasi tidak valid : {}", self.jalur.display()))?;

        pengaturan
            .validasi()
            .with_context(|| format!("Validasi konfigurasi gagal : {}", self.jalur.display()))?;

        Ok(pengaturan)
    }

    pub fn muat(&self) -> Result<PengaturanTerminal> {
        let isi = fs::read_to_string(&self.jalur)
            .with_context(|| format!("Gagal membaca konfigurasi : {}", self.jalur.display()))?;

        self.uraikan(&isi)
    }

    fn siapkan_berkas(&self, pengaturan: &PengaturanTerminal) -> Result<NamedTempFile> {
        pengaturan.validasi()?;

        let direktori = self
            .jalur
            .parent()
            .filter(|jalur| !jalur.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        fs::create_dir_all(direktori).with_context(|| {
            format!(
                "Gagal membuat direktori konfigurasi : {}",
                direktori.display()
            )
        })?;

        let isi = toml::to_string_pretty(pengaturan)
            .context("Gagal melakukan serialisasi konfigurasi")?;

        let mut sementara = NamedTempFile::new_in(direktori)
            .context("Gagal membuat berkas konfigurasi sementara")?;

        sementara
            .write_all(isi.as_bytes())
            .context("Gagal menulis konfigurasi")?;

        sementara
            .as_file()
            .sync_all()
            .context("Gagal menyinkronkan konfigurasi")?;

        Ok(sementara)
    }

    pub fn simpan(&self, pengaturan: &PengaturanTerminal) -> Result<()> {
        let sementara = self.siapkan_berkas(pengaturan)?;

        sementara
            .persist(&self.jalur)
            .with_context(|| format!("Gagal menyimpan konfigurasi : {}", self.jalur.display()))?;

        Ok(())
    }

    pub fn muat_atau_buat(&self) -> Result<PengaturanTerminal> {
        match fs::read_to_string(&self.jalur) {
            Ok(isi) => self.uraikan(&isi),

            Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => {
                let pengaturan = PengaturanTerminal::default();

                let sementara = self.siapkan_berkas(&pengaturan)?;

                match sementara.persist_noclobber(&self.jalur) {
                    Ok(_) => Ok(pengaturan),

                    Err(kesalahan) if kesalahan.error.kind() == ErrorKind::AlreadyExists => {
                        self.muat()
                    }

                    Err(kesalahan) => Err(kesalahan).with_context(|| {
                        format!("Gagal membuat konfigurasi : {}", self.jalur.display())
                    }),
                }
            }

            Err(kesalahan) => Err(kesalahan)
                .with_context(|| format!("Gagal membaca konfigurasi : {}", self.jalur.display())),
        }
    }
}
