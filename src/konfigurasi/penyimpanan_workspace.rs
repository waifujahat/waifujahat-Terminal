/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Workspace storage
 * Purpose : Loads and persists workspace state safely
 */

use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use tempfile::NamedTempFile;

use super::{PenyimpananPengaturan, RuangKerja};

const NAMA_BERKAS: &str = "workspace.toml";
const BATAS_UKURAN_BERKAS: usize = 1024 * 1024;

#[derive(Clone, Debug)]
pub struct PenyimpananRuangKerja {
    jalur: PathBuf,
}

impl PenyimpananRuangKerja {
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

    pub fn muat(&self) -> Result<Option<RuangKerja>> {
        let berkas = match File::open(&self.jalur) {
            Ok(berkas) => berkas,

            Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => {
                return Ok(None);
            }

            Err(kesalahan) => {
                return Err(kesalahan).with_context(|| {
                    format!("Gagal membuka workspace : {}", self.jalur.display())
                });
            }
        };

        let mut isi = Vec::new();

        berkas
            .take(BATAS_UKURAN_BERKAS as u64 + 1)
            .read_to_end(&mut isi)
            .with_context(|| format!("Gagal membaca workspace : {}", self.jalur.display()))?;

        ensure!(
            isi.len() <= BATAS_UKURAN_BERKAS,
            "Ukuran workspace melebihi batas 1 MiB"
        );

        let isi = std::str::from_utf8(&isi).context("Encoding workspace tidak valid")?;

        let isi = isi.strip_prefix('\u{feff}').unwrap_or(isi);

        let ruang: RuangKerja = toml::from_str(isi)
            .with_context(|| format!("Format workspace tidak valid : {}", self.jalur.display()))?;

        ruang
            .validasi()
            .with_context(|| format!("Validasi workspace gagal : {}", self.jalur.display()))?;

        Ok(Some(ruang))
    }

    pub fn simpan(&self, ruang: &RuangKerja) -> Result<()> {
        ruang.validasi()?;

        let isi = toml::to_string_pretty(ruang).context("Gagal melakukan serialisasi workspace")?;

        ensure!(
            isi.len() <= BATAS_UKURAN_BERKAS,
            "Ukuran workspace melebihi batas 1 MiB"
        );

        let direktori = self
            .jalur
            .parent()
            .filter(|jalur| !jalur.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        fs::create_dir_all(direktori).with_context(|| {
            format!(
                "Gagal membuat direktori workspace : {}",
                direktori.display()
            )
        })?;

        let mut sementara =
            NamedTempFile::new_in(direktori).context("Gagal membuat berkas workspace sementara")?;

        sementara
            .write_all(isi.as_bytes())
            .context("Gagal menulis workspace")?;

        sementara
            .as_file()
            .sync_all()
            .context("Gagal menyinkronkan workspace")?;

        sementara
            .persist(&self.jalur)
            .with_context(|| format!("Gagal menyimpan workspace : {}", self.jalur.display()))?;

        Ok(())
    }
}
