/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Workspace model
 * Purpose : Defines serializable workspace state and structural validation
 */

use std::path::PathBuf;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

use crate::terminal::{ArahPanel, BATAS_NAMA_TAB, SusunanPanel};

use super::PilihanShell;

const VERSI_RUANG_KERJA: u32 = 1;
pub(super) const BATAS_JUMLAH_TAB: usize = 64;
pub(super) const BATAS_JUMLAH_PANEL: usize = 256;
pub(super) const BATAS_KEDALAMAN_PANEL: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuangKerja {
    #[serde(rename = "version")]
    pub versi: u32,

    #[serde(rename = "active_tab")]
    pub tab_aktif: usize,

    #[serde(rename = "tabs")]
    pub tab: Vec<TabRuangKerja>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabRuangKerja {
    #[serde(rename = "name")]
    pub nama: String,

    #[serde(rename = "active_panel")]
    pub panel_aktif: usize,

    #[serde(rename = "layout")]
    pub susunan: PanelRuangKerja,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum PanelRuangKerja {
    #[serde(rename = "session")]
    Tunggal {
        shell: PilihanShell,

        #[serde(
            rename = "working_directory",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        direktori: Option<PathBuf>,
    },

    #[serde(rename = "split")]
    Terbagi {
        #[serde(rename = "direction")]
        arah: ArahPanel,

        #[serde(rename = "ratio")]
        rasio: u16,

        #[serde(rename = "first")]
        pertama: Box<PanelRuangKerja>,

        #[serde(rename = "second")]
        kedua: Box<PanelRuangKerja>,
    },
}

impl RuangKerja {
    pub const VERSI: u32 = VERSI_RUANG_KERJA;

    pub fn validasi(&self) -> Result<()> {
        ensure!(self.versi == Self::VERSI, "Versi workspace tidak didukung");

        ensure!(!self.tab.is_empty(), "Workspace tidak boleh kosong");

        ensure!(
            self.tab.len() <= BATAS_JUMLAH_TAB,
            "Jumlah tab workspace melebihi batas"
        );

        ensure!(
            self.tab_aktif < self.tab.len(),
            "Indeks tab aktif tidak valid"
        );

        let mut jumlah_panel = 0;

        for tab in &self.tab {
            tab.validasi(&mut jumlah_panel)?;
        }

        Ok(())
    }
}

impl TabRuangKerja {
    fn validasi(&self, jumlah_panel: &mut usize) -> Result<()> {
        let nama = self.nama.trim();

        ensure!(!nama.is_empty(), "Nama tab tidak boleh kosong");

        ensure!(self.nama == nama, "Nama tab workspace tidak valid");

        ensure!(
            nama.chars().count() <= BATAS_NAMA_TAB,
            "Nama tab melebihi batas panjang"
        );

        ensure!(
            !nama.chars().any(char::is_control),
            "Nama tab mengandung karakter kontrol"
        );

        let jumlah_sebelumnya = *jumlah_panel;

        self.susunan.validasi(0, jumlah_panel)?;

        let jumlah_panel_tab = *jumlah_panel - jumlah_sebelumnya;

        ensure!(
            self.panel_aktif < jumlah_panel_tab,
            "Indeks panel aktif tidak valid"
        );

        Ok(())
    }
}

impl PanelRuangKerja {
    fn validasi(&self, kedalaman: usize, jumlah_panel: &mut usize) -> Result<()> {
        ensure!(
            kedalaman <= BATAS_KEDALAMAN_PANEL,
            "Kedalaman split panel melebihi batas"
        );

        match self {
            Self::Tunggal { shell, direktori } => {
                ensure!(
                    shell.profil().is_some(),
                    "Profil shell workspace tidak valid"
                );

                if let Some(direktori) = direktori {
                    ensure!(
                        direktori.is_absolute(),
                        "Working directory harus menggunakan jalur absolut"
                    );
                }

                *jumlah_panel += 1;

                ensure!(
                    *jumlah_panel <= BATAS_JUMLAH_PANEL,
                    "Jumlah panel workspace melebihi batas"
                );
            }

            Self::Terbagi {
                rasio,
                pertama,
                kedua,
                ..
            } => {
                ensure!(
                    SusunanPanel::rasio_valid(*rasio),
                    "Rasio split panel tidak valid"
                );

                pertama.validasi(kedalaman + 1, jumlah_panel)?;
                kedua.validasi(kedalaman + 1, jumlah_panel)?;
            }
        }

        Ok(())
    }
}
