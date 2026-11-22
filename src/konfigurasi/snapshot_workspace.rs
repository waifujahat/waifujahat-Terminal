/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Workspace snapshot
 * Purpose : Captures runtime terminal state for persistent workspace restoration
 */

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

use crate::terminal::{IdSesi, ManajerTab, ProfilShell, SimpulPanel};

use super::{
    PanelRuangKerja, RuangKerja, TabRuangKerja,
    ruang_kerja::{BATAS_JUMLAH_PANEL, BATAS_JUMLAH_TAB, BATAS_KEDALAMAN_PANEL},
};

impl RuangKerja {
    pub fn dari_manajer<T>(
        manajer: &ManajerTab<T>,
        mut ambil_informasi: impl FnMut(&T) -> (ProfilShell, Option<PathBuf>),
    ) -> Result<Self> {
        ensure!(
            manajer.jumlah_tab() <= BATAS_JUMLAH_TAB,
            "Jumlah tab workspace melebihi batas"
        );

        ensure!(
            manajer.jumlah_sesi() <= BATAS_JUMLAH_PANEL,
            "Jumlah panel workspace melebihi batas"
        );

        let id_tab_aktif = manajer.id_tab_aktif().context("Tab aktif tidak tersedia")?;

        let mut tab_aktif = None;

        let mut daftar_tab = Vec::with_capacity(manajer.jumlah_tab());

        for (indeks, id) in manajer.daftar_tab().enumerate() {
            if id == id_tab_aktif {
                tab_aktif = Some(indeks);
            }

            let nama = manajer.nama_tab(id).context("Nama tab tidak tersedia")?;

            let susunan = manajer
                .ambil_tab(id)
                .context("Susunan panel tidak tersedia")?;

            let (snapshot, _, panel_aktif) = snapshot_panel(
                manajer,
                susunan.akar(),
                susunan.id_aktif(),
                0,
                &mut ambil_informasi,
            )?;

            daftar_tab.push(TabRuangKerja {
                nama: nama.to_owned(),
                panel_aktif: panel_aktif.context("Panel aktif tidak ditemukan")?,
                susunan: snapshot,
            });
        }

        let ruang = Self {
            versi: Self::VERSI,
            tab_aktif: tab_aktif.context("Indeks tab aktif tidak ditemukan")?,
            tab: daftar_tab,
        };

        ruang.validasi()?;

        Ok(ruang)
    }
}

fn snapshot_panel<T>(
    manajer: &ManajerTab<T>,
    simpul: &SimpulPanel,
    id_aktif: IdSesi,
    kedalaman: usize,
    ambil_informasi: &mut impl FnMut(&T) -> (ProfilShell, Option<PathBuf>),
) -> Result<(PanelRuangKerja, usize, Option<usize>)> {
    ensure!(
        kedalaman <= BATAS_KEDALAMAN_PANEL,
        "Kedalaman split panel melebihi batas"
    );

    match simpul {
        SimpulPanel::Tunggal(id) => {
            let sesi = manajer.ambil(*id).context("Sesi panel tidak ditemukan")?;

            let (profil, direktori) = ambil_informasi(sesi);

            Ok((
                PanelRuangKerja::Tunggal {
                    shell: profil.into(),
                    direktori,
                },
                1,
                (*id == id_aktif).then_some(0),
            ))
        }

        SimpulPanel::Terbagi {
            arah,
            rasio,
            pertama,
            kedua,
        } => {
            let (pertama, jumlah_pertama, aktif_pertama) =
                snapshot_panel(manajer, pertama, id_aktif, kedalaman + 1, ambil_informasi)?;

            let (kedua, jumlah_kedua, aktif_kedua) =
                snapshot_panel(manajer, kedua, id_aktif, kedalaman + 1, ambil_informasi)?;

            let panel_aktif = aktif_pertama.or(aktif_kedua.map(|indeks| indeks + jumlah_pertama));

            Ok((
                PanelRuangKerja::Terbagi {
                    arah: *arah,
                    rasio: *rasio,
                    pertama: Box::new(pertama),
                    kedua: Box::new(kedua),
                },
                jumlah_pertama + jumlah_kedua,
                panel_aktif,
            ))
        }
    }
}
