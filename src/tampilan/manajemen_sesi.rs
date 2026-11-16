/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session management
 * Purpose: Handles terminal session creation, navigation and closure
 */

use gpui::{Context, KeyDownEvent, Window};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

use waifujahat_terminal::terminal::IdSesi;

impl JendelaTerminal {
    pub(super) fn tambah_sesi(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        let mut ruang = RuangSesi::baru();

        if ruang.sesi.is_none() {
            self.ruang_mut().status = ruang.status;

            konteks.notify();
            return;
        }

        ruang.sesuaikan_ukuran(jendela, konteks);

        self.manajer.tambah(ruang);

        jendela.focus(&self.fokus, konteks);
        konteks.notify();
    }

    pub(super) fn pilih_sesi(
        &mut self,
        id: IdSesi,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.manajer.id_aktif() == Some(id) || self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        self.ruang_mut().keadaan.seleksi.selesai();
        self.ruang_mut().arah_gulir_seleksi = 0;

        if self.manajer.pilih(id) {
            jendela.focus(&self.fokus, konteks);
            konteks.notify();
        }
    }

    fn pindah_sesi(&mut self, mundur: bool, jendela: &mut Window, konteks: &mut Context<Self>) {
        if self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        self.ruang_mut().keadaan.seleksi.selesai();
        self.ruang_mut().arah_gulir_seleksi = 0;

        if self.manajer.geser_aktif(mundur) {
            jendela.focus(&self.fokus, konteks);
            konteks.notify();
        }
    }

    fn tutup_sesi(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        if let Some(id) = self.manajer.id_aktif() {
            self.tutup_sesi_dengan_id(id, jendela, konteks);
        }
    }

    pub(super) fn tutup_sesi_dengan_id(
        &mut self,
        id: IdSesi,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.manajer.jumlah() <= 1 {
            self.ruang_mut().status = "Sesi terakhir tidak dapat ditutup".into();

            konteks.notify();
            return;
        }

        let Some(ruang) = self.manajer.ambil(id) else {
            return;
        };

        if ruang.keadaan.siklus.berjalan() || ruang.keadaan.motor.layar_alternatif_aktif() {
            self.ruang_mut().status = "Hentikan perintah sebelum menutup sesi".into();

            konteks.notify();
            return;
        }

        drop(self.manajer.tutup(id));

        jendela.focus(&self.fokus, konteks);
        konteks.notify();
    }

    pub(super) fn tangani_tombol_sesi(
        &mut self,
        kejadian: &KeyDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> bool {
        let tombol = &kejadian.keystroke;

        if !tombol.modifiers.control || tombol.modifiers.alt || tombol.modifiers.platform {
            return false;
        }

        let nama = tombol.key.to_ascii_lowercase();

        match (nama.as_str(), tombol.modifiers.shift) {
            ("t", true) => {
                self.tambah_sesi(jendela, konteks);
            }

            ("w", true) => {
                self.tutup_sesi(jendela, konteks);
            }

            ("tab", false) => {
                self.pindah_sesi(false, jendela, konteks);
            }

            ("tab", true) | ("backtab", _) => {
                self.pindah_sesi(true, jendela, konteks);
            }

            _ => return false,
        }

        true
    }
}
