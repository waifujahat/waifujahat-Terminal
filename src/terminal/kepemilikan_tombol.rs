/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Keyboard ownership
 * Purpose: Tracks keyboard event ownership across terminal sessions
 */

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKepemilikanTombol<Id> {
    Dimiliki(Id),
    EditorNamaTab,
    Dibatalkan,
}

pub struct KepemilikanTombol<Id> {
    tombol: HashMap<String, StatusKepemilikanTombol<Id>>,
}

impl<Id: Copy + Eq> KepemilikanTombol<Id> {
    pub fn baru() -> Self {
        Self {
            tombol: HashMap::new(),
        }
    }

    pub fn tandai_editor_nama_tab(&mut self, nama: &str) {
        self.tombol.insert(
            nama.to_ascii_lowercase(),
            StatusKepemilikanTombol::EditorNamaTab,
        );
    }

    pub fn tandai_dibatalkan(&mut self, nama: &str) {
        self.tombol.insert(
            nama.to_ascii_lowercase(),
            StatusKepemilikanTombol::Dibatalkan,
        );
    }

    pub fn catat(&mut self, nama: &str, pemilik: Id) -> Option<StatusKepemilikanTombol<Id>> {
        self.tombol.insert(
            nama.to_ascii_lowercase(),
            StatusKepemilikanTombol::Dimiliki(pemilik),
        )
    }

    pub fn ambil(&self, nama: &str) -> Option<StatusKepemilikanTombol<Id>> {
        self.tombol.get(&nama.to_ascii_lowercase()).copied()
    }

    pub fn cari_awalan_unik(&self, awalan: &str) -> Option<String> {
        let mut ditemukan = self.tombol.keys().filter(|nama| nama.starts_with(awalan));

        let nama = ditemukan.next()?;

        if ditemukan.next().is_some() {
            return None;
        }

        Some(nama.clone())
    }

    pub fn lepas(&mut self, nama: &str) -> Option<StatusKepemilikanTombol<Id>> {
        self.tombol.remove(&nama.to_ascii_lowercase())
    }

    pub fn batalkan_pemilik(&mut self, pemilik: Id) {
        for status in self.tombol.values_mut() {
            if *status == StatusKepemilikanTombol::Dimiliki(pemilik) {
                *status = StatusKepemilikanTombol::Dibatalkan;
            }
        }
    }

    pub fn batalkan_semua(&mut self) {
        for status in self.tombol.values_mut() {
            *status = StatusKepemilikanTombol::Dibatalkan;
        }
    }
}
