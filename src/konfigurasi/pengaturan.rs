/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Terminal settings
 * Purpose : Defines serializable terminal preferences and defaults
 */

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::terminal::{ProfilShell, ProfilSsh};

use super::DaftarProfilSsh;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PilihanShell {
    #[default]
    #[serde(rename = "auto")]
    Otomatis,

    #[serde(rename = "windows-powershell")]
    WindowsPowerShell,

    #[serde(rename = "powershell-7")]
    PowerShell7,

    #[serde(rename = "cmd")]
    Cmd,

    #[serde(rename = "git-bash")]
    GitBash,
}

impl PilihanShell {
    pub fn profil(self) -> Option<ProfilShell> {
        match self {
            Self::Otomatis => None,
            Self::WindowsPowerShell => Some(ProfilShell::WindowsPowerShell),
            Self::PowerShell7 => Some(ProfilShell::PowerShell7),
            Self::Cmd => Some(ProfilShell::Cmd),
            Self::GitBash => Some(ProfilShell::GitBash),
        }
    }

    pub fn pilih_dengan(
        self,
        mut tersedia: impl FnMut(ProfilShell) -> bool,
    ) -> Option<ProfilShell> {
        self.profil()
            .filter(|profil| tersedia(*profil))
            .or_else(|| {
                ProfilShell::SEMUA
                    .into_iter()
                    .find(|profil| tersedia(*profil))
            })
    }

    pub fn profil_tersedia(self) -> Option<ProfilShell> {
        self.pilih_dengan(ProfilShell::tersedia)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PengaturanTerminal {
    #[serde(rename = "default_shell")]
    pub shell_default: PilihanShell,

    #[serde(
        rename = "default_ssh_profile",
        skip_serializing_if = "Option::is_none"
    )]
    pub ssh_default: Option<String>,

    #[serde(rename = "ssh_auto_connect")]
    pub ssh_auto_connect: bool,

    #[serde(rename = "font_family")]
    pub keluarga_font: String,

    #[serde(rename = "font_size", skip_serializing_if = "Option::is_none")]
    pub ukuran_font: Option<f32>,

    #[serde(rename = "window_width")]
    pub lebar_jendela: u32,

    #[serde(rename = "window_height")]
    pub tinggi_jendela: u32,

    #[serde(rename = "opacity", skip_serializing_if = "Option::is_none")]
    pub opasitas: Option<f32>,

    #[serde(rename = "wallpaper", skip_serializing_if = "Option::is_none")]
    pub gambar_latar: Option<PathBuf>,

    #[serde(rename = "scrollback", skip_serializing_if = "Option::is_none")]
    pub batas_riwayat: Option<usize>,

    #[serde(rename = "persistent_scrollback")]
    pub arsip_riwayat_aktif: bool,

    #[serde(rename = "confirmation")]
    pub konfirmasi_aktif: bool,
}

impl Default for PengaturanTerminal {
    fn default() -> Self {
        Self {
            shell_default: PilihanShell::Otomatis,
            keluarga_font: "Consolas".to_owned(),
            ukuran_font: None,
            lebar_jendela: 1120,
            tinggi_jendela: 630,
            opasitas: None,
            gambar_latar: None,
            batas_riwayat: None,
            arsip_riwayat_aktif: false,
            konfirmasi_aktif: true,
            ssh_default: None,
            ssh_auto_connect: false,
        }
    }
}

impl From<ProfilShell> for PilihanShell {
    fn from(profil: ProfilShell) -> Self {
        match profil {
            ProfilShell::WindowsPowerShell => Self::WindowsPowerShell,
            ProfilShell::PowerShell7 => Self::PowerShell7,
            ProfilShell::Cmd => Self::Cmd,
            ProfilShell::GitBash => Self::GitBash,
        }
    }
}

impl PengaturanTerminal {
    pub const UKURAN_FONT_DEFAULT: f32 = 14.0;
    pub const TINGGI_BARIS_DEFAULT: f32 = 18.0;
    pub const ALFA_LATAR_DEFAULT: u8 = 0xb3;

    pub fn profil_ssh_default<'a>(&self, daftar: &'a DaftarProfilSsh) -> Option<&'a ProfilSsh> {
        daftar.cari(self.ssh_default.as_deref()?)
    }

    pub fn ukuran_font_efektif(&self) -> f32 {
        self.ukuran_font.unwrap_or(Self::UKURAN_FONT_DEFAULT)
    }

    pub fn tinggi_baris(&self) -> f32 {
        (self.ukuran_font_efektif() * Self::TINGGI_BARIS_DEFAULT / Self::UKURAN_FONT_DEFAULT)
            .round()
    }

    pub fn tinggi_kursor(&self) -> f32 {
        (self.tinggi_baris() - 2.0).max(1.0)
    }

    pub fn alfa_latar(&self) -> u8 {
        self.opasitas.map_or(Self::ALFA_LATAR_DEFAULT, |nilai| {
            (nilai * 255.0).round() as u8
        })
    }
}
