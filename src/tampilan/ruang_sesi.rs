/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session workspace
 * Purpose: Owns terminal session and its presentation state
 */

use std::time::Instant;

use std::path::{Path, PathBuf};

use gpui::ScrollHandle;

use super::pencarian_terminal::PencarianPanel;

use std::collections::HashMap;

use super::keyboard_kitty::TombolKittyDitekan;

use waifujahat_terminal::konfigurasi::PengaturanTerminal;

use waifujahat_terminal::terminal::{
    JenisSesiTerminal, KeadaanSesi, ProfilShell, ProsesTerminal, RekamanTombolWin32, SesiTerminal,
};

#[derive(Default)]
pub(super) struct PenutupanSsh {
    pub(super) kode_keluar: Option<u32>,
    pub(super) pembaca_berakhir: bool,
    pub(super) batas: Option<Instant>,
    pub(super) kesalahan: Option<String>,
}

pub(super) struct RuangSesi {
    pub(super) jenis: JenisSesiTerminal,
    pub(super) penutupan_ssh: PenutupanSsh,
    pub(super) keadaan: KeadaanSesi,
    pub(super) sesi: Option<SesiTerminal>,
    pub(super) gulir: ScrollHandle,
    pub(super) status: String,
    pub(super) batas_status_salin: Option<Instant>,
    pub(super) arah_gulir_seleksi: i8,
    pub(super) kolom_gulir_seleksi: f32,
    pub(super) mouse_tui_ditekan: Option<(usize, usize)>,
    pub(super) ukuran_pty: (u16, u16),
    pub(super) ukuran_terakhir_diminta: Option<(u16, u16)>,
    pub(super) pencarian: PencarianPanel,
    pub(super) tombol_kitty_ditekan: HashMap<String, TombolKittyDitekan>,
    pub(super) tombol_win32_ditekan: HashMap<String, RekamanTombolWin32>,
    pub(super) shell_turunan: Vec<ProsesTerminal>,
    pub(super) pengaturan: PengaturanTerminal,
    direktori_awal: Option<PathBuf>,
}

impl RuangSesi {
    pub(super) fn baru() -> Self {
        let profil = ProfilShell::default_tersedia().unwrap_or_default();

        Self::baru_dengan_profil(profil)
    }

    pub(super) fn profil_lokal(&self) -> Option<ProfilShell> {
        match &self.jenis {
            JenisSesiTerminal::Lokal(profil) => Some(*profil),
            JenisSesiTerminal::Ssh(_) => None,
        }
    }

    pub(super) fn mendukung_integrasi_shell(&self) -> bool {
        self.profil_lokal()
            .is_some_and(ProfilShell::mendukung_integrasi_shell)
    }

    pub(super) fn baru_dengan_profil(profil: ProfilShell) -> Self {
        Self::baru_dengan_pengaturan(profil, &PengaturanTerminal::default())
    }

    pub(super) fn baru_dengan_pengaturan(
        profil: ProfilShell,
        pengaturan: &PengaturanTerminal,
    ) -> Self {
        Self::baru_dengan_direktori(profil, pengaturan, None)
    }

    pub(super) fn baru_dengan_direktori(
        profil: ProfilShell,
        pengaturan: &PengaturanTerminal,
        direktori: Option<&Path>,
    ) -> Self {
        Self::baru_dengan_jenis(JenisSesiTerminal::Lokal(profil), pengaturan, direktori)
    }

    pub(super) fn baru_dengan_jenis(
        jenis: JenisSesiTerminal,
        pengaturan: &PengaturanTerminal,
        direktori: Option<&Path>,
    ) -> Self {
        let hasil = match &jenis {
            JenisSesiTerminal::Lokal(profil) => {
                SesiTerminal::baru_dengan_direktori(*profil, direktori)
            }

            JenisSesiTerminal::Ssh(profil) => SesiTerminal::baru_dengan_ssh(profil),
        };

        Self::dari_hasil_sesi(jenis, pengaturan, hasil)
    }

    pub(super) fn dari_hasil_sesi(
        jenis: JenisSesiTerminal,
        pengaturan: &PengaturanTerminal,
        hasil: anyhow::Result<SesiTerminal>,
    ) -> Self {
        let mut keadaan = match &jenis {
            JenisSesiTerminal::Lokal(_) => KeadaanSesi::baru_dengan_pengaturan(
                80,
                24,
                pengaturan.konfirmasi_aktif,
                pengaturan.batas_riwayat,
            ),

            JenisSesiTerminal::Ssh(_) => {
                KeadaanSesi::baru_untuk_ssh(80, 24, pengaturan.batas_riwayat)
            }
        };

        let (sesi, status) = match hasil {
            Ok(sesi) => (Some(sesi), String::new()),

            Err(kesalahan) => {
                keadaan.siklus.akhiri_sesi();

                (
                    None,
                    format!("Gagal membuka {} : {kesalahan}", jenis.nama()),
                )
            }
        };

        let direktori_awal = sesi
            .as_ref()
            .and_then(SesiTerminal::direktori_awal)
            .map(Path::to_path_buf);

        Self {
            direktori_awal,
            pengaturan: pengaturan.clone(),
            jenis,
            penutupan_ssh: PenutupanSsh::default(),
            keadaan,
            sesi,
            gulir: ScrollHandle::new(),
            status,
            batas_status_salin: None,
            arah_gulir_seleksi: 0,
            kolom_gulir_seleksi: 0.0,
            mouse_tui_ditekan: None,
            ukuran_pty: (80, 24),
            ukuran_terakhir_diminta: None,
            pencarian: PencarianPanel::default(),
            tombol_kitty_ditekan: HashMap::new(),
            tombol_win32_ditekan: HashMap::new(),
            shell_turunan: Vec::new(),
        }
    }

    pub(super) fn direktori_workspace(&self) -> Option<&Path> {
        self.profil_lokal()?;

        self.keadaan
            .direktori_terakhir()
            .or(self.direktori_awal.as_deref())
    }
}
