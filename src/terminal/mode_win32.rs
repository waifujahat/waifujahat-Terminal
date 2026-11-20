/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Win32 input mode
 * Purpose: Tracks Win32 keyboard protocol negotiation
 */

use vte::{Params, Parser, Perform};

const MODE_INPUT_WIN32: u16 = 9001;

#[derive(Default)]
struct PenerimaModeWin32 {
    aktif: bool,
    generasi: u64,
}

impl PenerimaModeWin32 {
    fn atur(&mut self, aktif: bool) {
        if self.aktif != aktif {
            self.aktif = aktif;
            self.generasi = self.generasi.wrapping_add(1);
        }
    }
}

impl Perform for PenerimaModeWin32 {
    fn csi_dispatch(&mut self, parameter: &Params, perantara: &[u8], abaikan: bool, aksi: char) {
        if abaikan || perantara != b"?" || !matches!(aksi, 'h' | 'l') {
            return;
        }

        if parameter
            .iter()
            .any(|nilai| matches!(nilai, [MODE_INPUT_WIN32]))
        {
            self.atur(aksi == 'h');
        }
    }

    fn esc_dispatch(&mut self, perantara: &[u8], abaikan: bool, aksi: u8) {
        if !abaikan && perantara.is_empty() && aksi == b'c' {
            self.atur(false);
        }
    }
}

pub struct ModeInputWin32 {
    parser: Parser,
    penerima: PenerimaModeWin32,
}

impl Default for ModeInputWin32 {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
            penerima: PenerimaModeWin32::default(),
        }
    }
}

impl ModeInputWin32 {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn terima(&mut self, data: &[u8]) {
        self.parser.advance(&mut self.penerima, data);
    }

    pub fn aktif(&self) -> bool {
        self.penerima.aktif
    }

    pub fn generasi(&self) -> u64 {
        self.penerima.generasi
    }
}
