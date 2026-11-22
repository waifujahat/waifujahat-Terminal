/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / SSH storage
 * Purpose : Loads and persists SSH connection profiles safely
 */

use std::{
    fs::{self, File},
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use tempfile::NamedTempFile;

use super::{DaftarProfilSsh, PenyimpananPengaturan};

const NAMA_BERKAS: &str = "ssh.toml";
const BATAS_UKURAN: u64 = 1024 * 1024;

#[derive(Clone, Debug)]
pub struct PenyimpananSsh {
    jalur: PathBuf,
}

impl PenyimpananSsh {
    pub fn baru(jalur: impl Into<PathBuf>) -> Self {
        Self {
            jalur: jalur.into(),
        }
    }

    pub fn bawaan() -> Result<Self> {
        let pengaturan = PenyimpananPengaturan::bawaan()?;

        Ok(Self::baru(pengaturan.jalur().with_file_name(NAMA_BERKAS)))
    }

    pub fn jalur(&self) -> &Path {
        &self.jalur
    }

    pub fn muat(&self) -> Result<Option<DaftarProfilSsh>> {
        let berkas = match File::open(&self.jalur) {
            Ok(berkas) => berkas,

            Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => {
                return Ok(None);
            }

            Err(kesalahan) => {
                return Err(kesalahan).with_context(|| {
                    format!("Gagal membuka konfigurasi SSH : {}", self.jalur.display())
                });
            }
        };

        let mut data = Vec::new();

        berkas
            .take(BATAS_UKURAN + 1)
            .read_to_end(&mut data)
            .context("Gagal membaca konfigurasi SSH")?;

        ensure!(
            data.len() as u64 <= BATAS_UKURAN,
            "Ukuran konfigurasi SSH melebihi batas"
        );

        let isi = std::str::from_utf8(&data).context("Encoding konfigurasi SSH tidak valid")?;

        let isi = isi.strip_prefix('\u{feff}').unwrap_or(isi);

        let daftar: DaftarProfilSsh =
            toml::from_str(isi).context("Format konfigurasi SSH tidak valid")?;

        daftar.validasi()?;

        Ok(Some(daftar))
    }

    pub fn simpan(&self, daftar: &DaftarProfilSsh) -> Result<()> {
        daftar.validasi()?;

        let isi =
            toml::to_string_pretty(daftar).context("Gagal melakukan serialisasi profil SSH")?;

        ensure!(
            isi.len() as u64 <= BATAS_UKURAN,
            "Ukuran konfigurasi SSH melebihi batas"
        );

        // Jangan menimpa konfigurasi yang sudah rusak.
        let tersedia = self.muat()?.is_some();

        let direktori = self
            .jalur
            .parent()
            .filter(|jalur| !jalur.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        fs::create_dir_all(direktori).context("Gagal membuat direktori konfigurasi SSH")?;

        let mut sementara =
            NamedTempFile::new_in(direktori).context("Gagal membuat berkas sementara SSH")?;

        sementara
            .write_all(isi.as_bytes())
            .context("Gagal menulis konfigurasi SSH")?;

        sementara
            .as_file()
            .sync_all()
            .context("Gagal menyinkronkan konfigurasi SSH")?;

        let hasil = if tersedia {
            sementara.persist(&self.jalur)
        } else {
            sementara.persist_noclobber(&self.jalur)
        };

        hasil.with_context(|| {
            format!("Gagal menyimpan konfigurasi SSH : {}", self.jalur.display())
        })?;

        Ok(())
    }
}
