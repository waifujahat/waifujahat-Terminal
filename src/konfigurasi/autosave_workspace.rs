/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Workspace autosave
 * Purpose : Persists workspace changes and detects external modifications
 */

use anyhow::{Result, ensure};

use super::{PenyimpananRuangKerja, RuangKerja};

pub struct PenyimpanWorkspace {
    penyimpanan: PenyimpananRuangKerja,
    terakhir: Option<RuangKerja>,
}

impl PenyimpanWorkspace {
    pub fn baru(penyimpanan: PenyimpananRuangKerja) -> Result<Self> {
        let terakhir = penyimpanan.muat()?;

        Ok(Self {
            penyimpanan,
            terakhir,
        })
    }

    pub fn terakhir(&self) -> Option<&RuangKerja> {
        self.terakhir.as_ref()
    }

    pub fn simpan_jika_berubah(&mut self, workspace: &RuangKerja) -> Result<bool> {
        workspace.validasi()?;

        if self.terakhir.as_ref() == Some(workspace) {
            return Ok(false);
        }

        let aktual = self.penyimpanan.muat()?;

        ensure!(
            aktual == self.terakhir,
            "Workspace telah berubah secara eksternal"
        );

        self.penyimpanan.simpan(workspace)?;

        self.terakhir = Some(workspace.clone());

        Ok(true)
    }
}
