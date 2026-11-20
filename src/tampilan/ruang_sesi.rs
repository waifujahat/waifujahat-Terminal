/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session workspace
 * Purpose: Owns terminal session and its presentation state
 */

use std::time::Instant;

use gpui::ScrollHandle;

use waifujahat_terminal::terminal::{KeadaanSesi, RekamanTombolWin32, SesiTerminal};

use super::pencarian_terminal::PencarianPanel;

use std::collections::HashMap;

use super::keyboard_kitty::TombolKittyDitekan;

pub(super) struct RuangSesi {
    pub(super) keadaan: KeadaanSesi,
    pub(super) sesi: Option<SesiTerminal>,
    pub(super) gulir: ScrollHandle,
    pub(super) status: String,
    pub(super) batas_status_salin: Option<Instant>,
    pub(super) arah_gulir_seleksi: i8,
    pub(super) kolom_gulir_seleksi: usize,
    pub(super) mouse_tui_ditekan: Option<(usize, usize)>,
    pub(super) ukuran_pty: (u16, u16),
    pub(super) ukuran_terakhir_diminta: Option<(u16, u16)>,
    pub(super) pencarian: PencarianPanel,
    pub(super) tombol_kitty_ditekan: HashMap<String, TombolKittyDitekan>,
    pub(super) tombol_win32_ditekan: HashMap<String, RekamanTombolWin32>,
}

impl RuangSesi {
    pub(super) fn baru() -> Self {
        let mut keadaan = KeadaanSesi::baru(80, 24);

        let (sesi, status) = match SesiTerminal::baru() {
            Ok(sesi) => (Some(sesi), String::new()),

            Err(kesalahan) => {
                keadaan.siklus.akhiri_sesi();

                (None, format!("Gagal membuka PowerShell: {kesalahan}"))
            }
        };

        Self {
            keadaan,
            sesi,
            gulir: ScrollHandle::new(),
            status,
            batas_status_salin: None,
            arah_gulir_seleksi: 0,
            kolom_gulir_seleksi: 0,
            mouse_tui_ditekan: None,
            ukuran_pty: (80, 24),
            ukuran_terakhir_diminta: None,
            pencarian: PencarianPanel::default(),
            tombol_kitty_ditekan: HashMap::new(),
            tombol_win32_ditekan: HashMap::new(),
        }
    }
}
