/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Kitty keyboard mode
 * Purpose: Manages bounded keyboard protocol mode stacks
 */

use std::collections::VecDeque;

const BATAS_STACK: usize = 32;
const FLAG_DIDUKUNG: u16 = 0b00011;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PengaturanModeKitty {
    Ganti,
    Tambah,
    Hapus,
}

#[derive(Default)]
pub struct ModeKeyboardKitty {
    flag: u16,
    stack: VecDeque<u16>,
}

impl ModeKeyboardKitty {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn flag(&self) -> u16 {
        self.flag
    }

    pub fn kedalaman(&self) -> usize {
        self.stack.len()
    }

    pub fn disambiguasi_aktif(&self) -> bool {
        self.flag & 1 != 0
    }

    pub fn pelaporan_aksi_aktif(&self) -> bool {
        self.flag & 2 != 0
    }

    pub fn atur(&mut self, flag: u16, pengaturan: PengaturanModeKitty) {
        let flag = flag & FLAG_DIDUKUNG;

        self.flag = match pengaturan {
            PengaturanModeKitty::Ganti => flag,
            PengaturanModeKitty::Tambah => self.flag | flag,
            PengaturanModeKitty::Hapus => self.flag & !flag,
        };
    }

    pub fn dorong(&mut self, flag: u16) {
        if self.stack.len() >= BATAS_STACK {
            self.stack.pop_front();
        }

        self.stack.push_back(self.flag);
        self.flag = flag & FLAG_DIDUKUNG;
    }

    pub fn keluarkan(&mut self, jumlah: usize) {
        if jumlah == 0 {
            return;
        }

        if jumlah > self.stack.len() {
            self.reset();
            return;
        }

        for _ in 0..jumlah {
            self.flag = self.stack.pop_back().unwrap_or_default();
        }
    }

    pub fn respons(&self) -> Vec<u8> {
        format!("\x1b[?{}u", self.flag).into_bytes()
    }

    pub fn reset(&mut self) {
        self.flag = 0;
        self.stack.clear();
    }
}
