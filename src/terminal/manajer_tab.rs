/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Tab manager
 * Purpose : Coordinates terminal tabs, pane layouts and independent session ownership
 */

use std::collections::HashMap;

use super::manajer_sesi::{IdSesi, ManajerSesi};
use super::susunan_panel::{ArahPanel, CabangPanel, SusunanPanel};

pub const BATAS_NAMA_TAB: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IdTab(IdSesi);

impl IdTab {
    pub fn nilai(self) -> u64 {
        self.0.nilai()
    }
}

pub struct ManajerTab<T> {
    tab: ManajerSesi<SusunanPanel>,
    sesi: ManajerSesi<T>,
    nama_tab: HashMap<IdTab, String>,
}

impl<T> ManajerTab<T> {
    pub fn baru() -> Self {
        Self {
            tab: ManajerSesi::baru(),
            sesi: ManajerSesi::baru(),
            nama_tab: HashMap::new(),
        }
    }

    pub fn jumlah_tab(&self) -> usize {
        self.tab.jumlah()
    }

    pub fn jumlah_sesi(&self) -> usize {
        self.sesi.jumlah()
    }

    pub fn id_tab_aktif(&self) -> Option<IdTab> {
        self.tab.id_aktif().map(IdTab)
    }

    pub fn id_panel_aktif(&self) -> Option<IdSesi> {
        self.tab.aktif().map(SusunanPanel::id_aktif)
    }

    pub fn daftar_tab(&self) -> impl Iterator<Item = IdTab> + '_ {
        self.tab.daftar_id().map(IdTab)
    }

    pub fn ambil_tab(&self, id: IdTab) -> Option<&SusunanPanel> {
        self.tab.ambil(id.0)
    }

    pub fn tab_aktif(&self) -> Option<&SusunanPanel> {
        self.tab.aktif()
    }

    pub fn ambil(&self, id: IdSesi) -> Option<&T> {
        self.sesi.ambil(id)
    }

    pub fn ambil_mut(&mut self, id: IdSesi) -> Option<&mut T> {
        self.sesi.ambil_mut(id)
    }

    pub fn aktif(&self) -> Option<&T> {
        self.ambil(self.id_panel_aktif()?)
    }

    pub fn aktif_mut(&mut self) -> Option<&mut T> {
        self.ambil_mut(self.id_panel_aktif()?)
    }

    pub fn iter_sesi_mut(&mut self) -> impl Iterator<Item = (IdSesi, &mut T)> + '_ {
        self.sesi.iter_mut()
    }

    pub fn tambah_tab(&mut self, isi: T) -> IdTab {
        let id_sesi = self.sesi.tambah(isi);

        let id = IdTab(self.tab.tambah(SusunanPanel::baru(id_sesi)));

        self.nama_tab
            .insert(id, format!("waifujahat {}", id.nilai()));

        id
    }

    pub fn nama_tab(&self, id: IdTab) -> Option<&str> {
        self.nama_tab.get(&id).map(String::as_str)
    }

    pub fn ubah_nama_tab(&mut self, id: IdTab, nama: &str) -> bool {
        if nama.chars().any(char::is_control) {
            return false;
        }

        let nama = nama.trim();

        if nama.is_empty() || nama.chars().count() > BATAS_NAMA_TAB {
            return false;
        }

        let Some(nama_tab) = self.nama_tab.get_mut(&id) else {
            return false;
        };

        *nama_tab = nama.to_owned();

        true
    }

    pub fn pilih_tab(&mut self, id: IdTab) -> bool {
        self.tab.pilih(id.0)
    }

    pub fn geser_tab(&mut self, mundur: bool) -> bool {
        self.tab.geser_aktif(mundur)
    }

    pub fn pilih_panel(&mut self, id: IdSesi) -> bool {
        self.tab
            .aktif_mut()
            .is_some_and(|susunan| susunan.pilih(id))
    }

    pub fn pecah_aktif(&mut self, isi: T, arah: ArahPanel) -> Option<IdSesi> {
        let susunan = self.tab.aktif_mut()?;
        let id = self.sesi.tambah(isi);

        assert!(susunan.pecah(id, arah), "Gagal membagi panel aktif");

        Some(id)
    }

    pub fn atur_rasio_panel(&mut self, id: IdTab, jalur: &[CabangPanel], rasio: u16) -> bool {
        if self.id_tab_aktif() != Some(id) {
            return false;
        }

        self.tab
            .aktif_mut()
            .is_some_and(|susunan| susunan.atur_rasio(jalur, rasio))
    }

    pub fn tutup_panel(&mut self, id: IdSesi) -> Option<T> {
        let susunan = self.tab.aktif_mut()?;

        if !susunan.tutup(id) {
            return None;
        }

        self.sesi.tutup(id)
    }

    pub fn tutup_tab(&mut self, id: IdTab) -> Option<Vec<T>> {
        let susunan = self.tab.tutup(id.0)?;

        self.nama_tab.remove(&id);

        let sesi = susunan
            .daftar_id()
            .into_iter()
            .map(|id| self.sesi.tutup(id).expect("Sesi panel tidak ditemukan"))
            .collect();

        Some(sesi)
    }
}
