/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / History archive storage
 * Purpose: Allocates isolated persistent history directories per session
 */

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use tempfile::Builder;

use crate::terminal::{
    ArsipRiwayat, PengarsipSesi, PengendaliAliranArsip, PengendaliArsip, PenulisArsip,
};

const NAMA_DIREKTORI: &str = "Waifujahat Terminal";
const NAMA_ARSIP: &str = "scrollback";

#[derive(Clone, Debug)]
pub struct PenyimpananArsip {
    akar: PathBuf,
}

impl PenyimpananArsip {
    pub fn baru(akar: impl Into<PathBuf>) -> Self {
        Self { akar: akar.into() }
    }

    pub fn bawaan() -> Result<Self> {
        let direktori = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .context("Variabel lingkungan LOCALAPPDATA tidak tersedia")?;

        ensure!(
            direktori.is_absolute(),
            "Direktori LOCALAPPDATA tidak valid"
        );

        Ok(Self::baru(direktori.join(NAMA_DIREKTORI).join(NAMA_ARSIP)))
    }

    pub fn akar(&self) -> &Path {
        &self.akar
    }

    pub fn buat_pengendali_aliran(
        &self,
        aktif: bool,
        batas_riwayat: Option<usize>,
    ) -> Result<Option<PengendaliAliranArsip>> {
        if !aktif {
            return Ok(None);
        }

        let kapasitas =
            batas_riwayat.context("Persistent scrollback memerlukan scrollback numerik")?;

        ensure!(
            kapasitas >= 4,
            "Persistent scrollback memerlukan scrollback minimal empat baris"
        );

        let arsip = self
            .mulai_sesi(true)?
            .context("Direktori arsip sesi tidak tersedia")?;

        let penulis = PenulisArsip::baru(arsip)?;
        let pengendali = PengendaliArsip::baru(penulis);
        let pengarsip = PengarsipSesi::baru(pengendali);

        Ok(Some(PengendaliAliranArsip::baru(pengarsip, kapasitas)?))
    }

    pub fn mulai_sesi(&self, aktif: bool) -> Result<Option<ArsipRiwayat>> {
        if !aktif {
            return Ok(None);
        }

        ensure!(
            self.akar.is_absolute(),
            "Direktori penyimpanan arsip harus absolut"
        );

        fs::create_dir_all(&self.akar).context("Gagal membuat direktori penyimpanan arsip")?;

        let sementara = Builder::new()
            .prefix("session-")
            .tempdir_in(&self.akar)
            .context("Gagal membuat direktori arsip sesi")?;
        let arsip = ArsipRiwayat::baru(sementara.path())?;

        let _jalur_permanen = sementara.keep();

        Ok(Some(arsip))
    }
}
