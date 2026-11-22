/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Workspace management
 * Purpose : Restores and persists active terminal workspaces
 */

use std::{path::Path, sync::Mutex};

use anyhow::{Result, ensure};

use waifujahat_terminal::{
    konfigurasi::{
        DaftarProfilSsh, PengaturanTerminal, PenyimpanWorkspace, PenyimpananSsh, RuangKerja,
    },
    terminal::{JenisSesiTerminal, ManajerTab},
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
        let bawaan = PengaturanTerminal::default();

        let pengaturan_workspace = pengaturan.unwrap_or(&bawaan);

        if let Some(workspace) = workspace {
            let hasil = (|| {
                workspace.validasi()?;

                let daftar_ssh = if workspace.memiliki_ssh() {
                    PenyimpananSsh::bawaan()?.muat()?.unwrap_or_default()
                } else {
                    DaftarProfilSsh::default()
                };

                workspace
                    .bangun_manajer_dengan_jenis(&daftar_ssh, |jenis, direktori| {
                        let ruang =
                            RuangSesi::baru_dengan_jenis(jenis, pengaturan_workspace, direktori);

                        ensure!(ruang.sesi.is_some(), "{}", ruang.status);

                        Ok(ruang)
                    })
                    .and_then(|manajer| {
                        Self::snapshot_workspace(&manajer)?;

                        Ok(manajer)
                    })
            })();

            match hasil {
                Ok(manajer) => return (manajer, true),

                Err(kesalahan) => {
                    eprintln!("Gagal memulihkan workspace : {kesalahan:#}");
                    eprintln!("Menggunakan sesi terminal bawaan");
                }
            }
        }

        let mut kesalahan_ssh = None;

        if workspace.is_none() && pengaturan_workspace.ssh_auto_connect {
            match Self::muat_profil_ssh_default(pengaturan_workspace) {
                Ok(Some(profil)) => {
                    let nama = profil.nama.clone();

                    let ruang = RuangSesi::baru_dengan_jenis(
                        JenisSesiTerminal::Ssh(profil),
                        pengaturan_workspace,
                        None,
                    );

                    if ruang.sesi.is_some() {
                        let mut manajer = ManajerTab::baru();

                        let id_tab = manajer.tambah_tab(ruang);

                        manajer.ubah_nama_tab(id_tab, &nama);

                        return (manajer, true);
                    }

                    kesalahan_ssh = Some(ruang.status);
                }

                Ok(None) => {
                    kesalahan_ssh = Some("Profil SSH default belum ditentukan".to_owned());
                }

                Err(kesalahan) => {
                    kesalahan_ssh = Some(format!("{kesalahan:#}"));
                }
            }
        }

        let mut ruang = match pengaturan {
            Some(pengaturan) => {
                let profil = pengaturan
                    .shell_default
                    .profil_tersedia()
                    .unwrap_or_default();

                RuangSesi::baru_dengan_pengaturan(profil, pengaturan)
            }

            None => RuangSesi::baru(),
        };

        if ruang.sesi.is_some()
            && let Some(pesan) = kesalahan_ssh
        {
            ruang.status = format!("Menggunakan shell lokal | {pesan}");
        }

        let aman_disimpan = workspace.is_none() && ruang.sesi.is_some();

        let mut manajer = ManajerTab::baru();

        manajer.tambah_tab(ruang);

        (manajer, aman_disimpan)
    }

    pub(super) fn snapshot_workspace(manajer: &ManajerTab<RuangSesi>) -> Result<RuangKerja> {
        RuangKerja::dari_manajer_dengan_jenis(manajer, |ruang| {
            let direktori = match &ruang.jenis {
                JenisSesiTerminal::Lokal(_) => ruang.direktori_workspace().map(Path::to_path_buf),

                JenisSesiTerminal::Ssh(_) => None,
            };

            (ruang.jenis.clone(), direktori)
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
