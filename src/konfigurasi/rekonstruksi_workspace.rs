/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Workspace reconstruction
 * Purpose : Reconstructs terminal sessions and recursive pane layouts
 */

use std::path::Path;

use anyhow::{Context, Result, ensure};

use crate::terminal::{CabangPanel, IdSesi, IdTab, JenisSesiTerminal, ManajerTab, ProfilShell};

use super::{DaftarProfilSsh, PanelRuangKerja, RuangKerja};

impl RuangKerja {
    pub fn bangun_manajer<T>(
        &self,
        mut buat_sesi: impl FnMut(ProfilShell, Option<&Path>) -> Result<T>,
    ) -> Result<ManajerTab<T>> {
        self.bangun_manajer_dengan_jenis(
            &DaftarProfilSsh::default(),
            |jenis, direktori| match jenis {
                JenisSesiTerminal::Lokal(profil) => buat_sesi(profil, direktori),

                JenisSesiTerminal::Ssh(_) => {
                    anyhow::bail!("Profil SSH workspace tidak tersedia")
                }
            },
        )
    }

    pub fn bangun_manajer_dengan_jenis<T>(
        &self,
        daftar_ssh: &DaftarProfilSsh,
        mut buat_sesi: impl FnMut(JenisSesiTerminal, Option<&Path>) -> Result<T>,
    ) -> Result<ManajerTab<T>> {
        self.validasi()?;

        daftar_ssh.validasi()?;

        for tab in &self.tab {
            validasi_profil_panel(&tab.susunan, daftar_ssh)?;
        }

        let mut manajer = ManajerTab::baru();

        let mut id_tab_aktif = None;

        for (indeks, tab) in self.tab.iter().enumerate() {
            let sesi = buat_sesi_pertama(&tab.susunan, daftar_ssh, &mut buat_sesi)?;

            let id_tab = manajer.tambah_tab(sesi);

            let id_panel = manajer
                .id_panel_aktif()
                .context("Panel awal tidak tersedia")?;

            ensure!(
                manajer.ubah_nama_tab(id_tab, &tab.nama),
                "Gagal memulihkan nama tab"
            );

            bangun_panel(
                &mut manajer,
                id_tab,
                id_panel,
                &tab.susunan,
                &mut Vec::new(),
                daftar_ssh,
                &mut buat_sesi,
            )?;

            let susunan = manajer
                .ambil_tab(id_tab)
                .context("Susunan panel tidak tersedia")?;

            let panel_aktif = susunan
                .daftar_id()
                .get(tab.panel_aktif)
                .copied()
                .context("Panel aktif workspace tidak ditemukan")?;

            ensure!(
                manajer.pilih_panel(panel_aktif),
                "Gagal memulihkan panel aktif"
            );

            if indeks == self.tab_aktif {
                id_tab_aktif = Some(id_tab);
            }
        }

        let id_tab_aktif = id_tab_aktif.context("Tab aktif workspace tidak ditemukan")?;

        ensure!(
            manajer.pilih_tab(id_tab_aktif),
            "Gagal memulihkan tab aktif"
        );

        Ok(manajer)
    }
}

fn validasi_profil_panel(panel: &PanelRuangKerja, daftar_ssh: &DaftarProfilSsh) -> Result<()> {
    match panel {
        PanelRuangKerja::Tunggal { .. } => {}

        PanelRuangKerja::Ssh { id_profil } => {
            ensure!(
                daftar_ssh.cari(id_profil).is_some(),
                "Profil SSH workspace tidak ditemukan : {id_profil}"
            );
        }

        PanelRuangKerja::Terbagi { pertama, kedua, .. } => {
            validasi_profil_panel(pertama, daftar_ssh)?;

            validasi_profil_panel(kedua, daftar_ssh)?;
        }
    }

    Ok(())
}

fn buat_sesi_pertama<T>(
    panel: &PanelRuangKerja,
    daftar_ssh: &DaftarProfilSsh,
    buat_sesi: &mut impl FnMut(JenisSesiTerminal, Option<&Path>) -> Result<T>,
) -> Result<T> {
    match panel {
        PanelRuangKerja::Tunggal { shell, direktori } => {
            let profil = shell
                .profil()
                .context("Profil shell workspace tidak valid")?;

            buat_sesi(JenisSesiTerminal::Lokal(profil), direktori.as_deref())
                .with_context(|| format!("Gagal memulihkan sesi : {}", profil.nama()))
        }

        PanelRuangKerja::Ssh { id_profil } => {
            let profil = daftar_ssh
                .cari(id_profil)
                .with_context(|| format!("Profil SSH workspace tidak ditemukan : {id_profil}"))?;

            buat_sesi(JenisSesiTerminal::Ssh(profil.clone()), None)
                .with_context(|| format!("Gagal memulihkan sesi SSH : {id_profil}"))
        }

        PanelRuangKerja::Terbagi { pertama, .. } => {
            buat_sesi_pertama(pertama, daftar_ssh, buat_sesi)
        }
    }
}

fn bangun_panel<T>(
    manajer: &mut ManajerTab<T>,
    id_tab: IdTab,
    id_panel: IdSesi,
    panel: &PanelRuangKerja,
    jalur: &mut Vec<CabangPanel>,
    daftar_ssh: &DaftarProfilSsh,
    buat_sesi: &mut impl FnMut(JenisSesiTerminal, Option<&Path>) -> Result<T>,
) -> Result<()> {
    let PanelRuangKerja::Terbagi {
        arah,
        rasio,
        pertama,
        kedua,
    } = panel
    else {
        return Ok(());
    };

    ensure!(
        manajer.pilih_panel(id_panel),
        "Gagal memilih panel untuk rekonstruksi"
    );

    let sesi = buat_sesi_pertama(kedua, daftar_ssh, buat_sesi)?;

    let id_kedua = manajer
        .pecah_aktif(sesi, *arah)
        .context("Gagal memulihkan split panel")?;

    // Rasio bawaan dapat menghasilkan false
    // karena nilainya sudah sesuai.

    manajer.atur_rasio_panel(id_tab, jalur, *rasio);

    jalur.push(CabangPanel::Pertama);

    bangun_panel(
        manajer, id_tab, id_panel, pertama, jalur, daftar_ssh, buat_sesi,
    )?;

    jalur.pop();

    jalur.push(CabangPanel::Kedua);

    bangun_panel(
        manajer, id_tab, id_kedua, kedua, jalur, daftar_ssh, buat_sesi,
    )?;

    jalur.pop();

    Ok(())
}
