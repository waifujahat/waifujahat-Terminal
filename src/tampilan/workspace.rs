/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Workspace management
 * Purpose : Restores and persists active terminal workspaces
 */

use std::{path::Path, sync::Mutex};

use anyhow::{Result, ensure};

use waifujahat_terminal::{
    konfigurasi::{PengaturanTerminal, PenyimpanWorkspace, RuangKerja},
    terminal::ManajerTab,
};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

struct KeadaanAutosave {
    penyimpan: PenyimpanWorkspace,
    ditutup: bool,
    kesalahan_terakhir: Option<String>,
}

pub(super) struct AutosaveWorkspace {
    keadaan: Mutex<KeadaanAutosave>,
}

impl AutosaveWorkspace {
    pub(super) fn baru(penyimpan: PenyimpanWorkspace) -> Self {
        Self {
            keadaan: Mutex::new(KeadaanAutosave {
                penyimpan,
                ditutup: false,
                kesalahan_terakhir: None,
            }),
        }
    }

    pub(super) fn simpan(&self, snapshot: Result<RuangKerja>, tutup: bool) {
        let Ok(mut keadaan) = self.keadaan.lock() else {
            return;
        };

        if keadaan.ditutup {
            return;
        }

        if tutup {
            keadaan.ditutup = true;
        }

        let hasil =
            snapshot.and_then(|workspace| keadaan.penyimpan.simpan_jika_berubah(&workspace));

        match hasil {
            Ok(_) => {
                keadaan.kesalahan_terakhir = None;
            }

            Err(kesalahan) => {
                let pesan = format!("{kesalahan:#}");

                if keadaan.kesalahan_terakhir.as_ref() != Some(&pesan) {
                    eprintln!("Gagal menyimpan workspace : {pesan}");

                    keadaan.kesalahan_terakhir = Some(pesan);
                }
            }
        }
    }
}

impl JendelaTerminal {
    pub(super) fn siapkan_manajer(
        pengaturan: Option<&PengaturanTerminal>,
        workspace: Option<&RuangKerja>,
    ) -> (ManajerTab<RuangSesi>, bool) {
        if let Some(workspace) = workspace {
            let bawaan = PengaturanTerminal::default();

            let pengaturan_workspace = pengaturan.unwrap_or(&bawaan);

            let hasil = workspace
                .bangun_manajer(|profil, direktori| {
                    let ruang =
                        RuangSesi::baru_dengan_direktori(profil, pengaturan_workspace, direktori);

                    ensure!(ruang.sesi.is_some(), "{}", ruang.status);

                    Ok(ruang)
                })
                .and_then(|manajer| {
                    Self::snapshot_workspace(&manajer)?;

                    Ok(manajer)
                });

            match hasil {
                Ok(manajer) => return (manajer, true),

                Err(kesalahan) => {
                    eprintln!("Gagal memulihkan workspace : {kesalahan:#}");

                    eprintln!("Menggunakan sesi terminal bawaan");
                }
            }
        }

        let ruang = match pengaturan {
            Some(pengaturan) => {
                let profil = pengaturan
                    .shell_default
                    .profil_tersedia()
                    .unwrap_or_default();

                RuangSesi::baru_dengan_pengaturan(profil, pengaturan)
            }

            None => RuangSesi::baru(),
        };

        let aman_disimpan = workspace.is_none() && ruang.sesi.is_some();

        let mut manajer = ManajerTab::baru();

        manajer.tambah_tab(ruang);

        (manajer, aman_disimpan)
    }

    pub(super) fn snapshot_workspace(manajer: &ManajerTab<RuangSesi>) -> Result<RuangKerja> {
        RuangKerja::dari_manajer(manajer, |ruang| {
            let direktori = ruang.direktori_workspace().map(Path::to_path_buf);

            (ruang.profil, direktori)
        })
    }

    pub(super) fn simpan_workspace_akhir(&mut self) {
        let Some(penyimpan) = self.penyimpan_workspace.take() else {
            return;
        };

        penyimpan.simpan(Self::snapshot_workspace(&self.manajer), true);
    }
}

impl Drop for JendelaTerminal {
    fn drop(&mut self) {
        self.simpan_workspace_akhir();
    }
}
