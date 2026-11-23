/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session management
 * Purpose: Handles terminal tab creation, navigation and closure
 */

use gpui::{Context, KeyDownEvent, Window};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

use waifujahat_terminal::{
    konfigurasi::{PengaturanTerminal, PenyimpananSsh},
    terminal::{
        ArahPanel, IdSesi, IdTab, JenisSesiTerminal, PengendaliOutputArsip, ProfilShell, ProfilSsh,
    },
};

impl JendelaTerminal {
    pub(super) fn jadwalkan_penyelesaian_arsip(
        &mut self,
        pengendali: PengendaliOutputArsip,
        snapshot: Option<Vec<String>>,
    ) {
        match pengendali.mulai_penyelesaian_dengan_snapshot(snapshot) {
            Ok(tugas) => {
                self.tugas_penyelesaian_arsip.push(tugas);
            }

            Err(kesalahan) => {
                eprintln!("Gagal menjadwalkan penyelesaian arsip: {kesalahan:#}");

                if let Some(ruang) = self.manajer.aktif_mut() {
                    ruang.status = format!("Gagal menjadwalkan penyelesaian arsip: {kesalahan:#}");
                }
            }
        }
    }

    pub(super) fn muat_profil_ssh_default(
        pengaturan: &PengaturanTerminal,
    ) -> anyhow::Result<Option<ProfilSsh>> {
        let Some(id) = &pengaturan.ssh_default else {
            return Ok(None);
        };

        let daftar = PenyimpananSsh::bawaan()?.muat()?.unwrap_or_default();

        let profil = pengaturan
            .profil_ssh_default(&daftar)
            .ok_or_else(|| anyhow::anyhow!("Profil SSH default tidak ditemukan : {id}"))?;

        Ok(Some(profil.clone()))
    }

    pub(super) fn tambah_sesi(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        let kesalahan = match Self::muat_profil_ssh_default(&self.pengaturan) {
            Ok(Some(profil)) => {
                if self.tambah_sesi_dengan_jenis(JenisSesiTerminal::Ssh(profil), jendela, konteks) {
                    return;
                }

                Some(self.ruang().status.clone())
            }

            Ok(None) => None,

            Err(kesalahan) => Some(format!("{kesalahan:#}")),
        };

        let profil = self
            .pengaturan
            .shell_default
            .profil_tersedia()
            .unwrap_or_default();

        if self.tambah_sesi_dengan_profil(profil, jendela, konteks)
            && let Some(pesan) = kesalahan
        {
            self.ruang_mut().status = format!("Menggunakan shell lokal | {pesan}");

            konteks.notify();
        }
    }

    pub(super) fn tambah_sesi_dengan_profil(
        &mut self,
        profil: ProfilShell,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> bool {
        self.tambah_sesi_dengan_jenis(JenisSesiTerminal::Lokal(profil), jendela, konteks)
    }

    pub(super) fn tambah_sesi_dengan_jenis(
        &mut self,
        jenis: JenisSesiTerminal,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> bool {
        let nama_tab = match &jenis {
            JenisSesiTerminal::Lokal(_) => None,

            JenisSesiTerminal::Ssh(profil) => Some(profil.nama.clone()),
        };

        let ruang = RuangSesi::baru_dengan_jenis(jenis, &self.pengaturan, None);

        if ruang.sesi.is_none() {
            self.ruang_mut().status = ruang.status;

            konteks.notify();

            return false;
        }

        let id_tab = self.manajer.tambah_tab(ruang);

        if let Some(nama) = nama_tab {
            self.manajer.ubah_nama_tab(id_tab, &nama);
        }

        jendela.focus(&self.fokus, konteks);

        konteks.notify();

        true
    }

    pub(super) fn pecah_panel(
        &mut self,
        arah: ArahPanel,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        let jenis = self.ruang().jenis.clone();

        let ruang = RuangSesi::baru_dengan_jenis(jenis, &self.pengaturan, None);

        if ruang.sesi.is_none() {
            self.ruang_mut().status = ruang.status;

            konteks.notify();
            return;
        }

        self.ruang_mut().keadaan.seleksi.selesai();
        self.ruang_mut().arah_gulir_seleksi = 0;

        if self.manajer.pecah_aktif(ruang, arah).is_some() {
            jendela.focus(&self.fokus, konteks);

            konteks.notify();
        }
    }

    pub(super) fn tutup_panel_dengan_id(
        &mut self,
        id: IdSesi,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        let Some(susunan) = self.manajer.tab_aktif() else {
            return;
        };

        if susunan.jumlah() <= 1 || !susunan.daftar_id().contains(&id) {
            return;
        }

        let sesi_sibuk = self.manajer.ambil(id).is_some_and(|ruang| {
            ruang.sesi.is_some()
                && (ruang.keadaan.siklus.berjalan()
                    || ruang.keadaan.motor.layar_alternatif_aktif()
                    || ruang.mouse_tui_ditekan.is_some())
        });

        if sesi_sibuk {
            self.ruang_mut().status = "Hentikan perintah sebelum menutup panel".into();

            konteks.notify();
            return;
        }

        if self
            .manajer
            .ambil_mut(id)
            .is_some_and(RuangSesi::ada_output_arsip_tertunda)
        {
            self.ruang_mut().status =
                "Selesaikan sesi dengan exit dan tunggu pengarsipan sebelum menutup panel"
                    .to_owned();

            konteks.notify();
            return;
        }

        if let Some(mut ruang) = self.manajer.tutup_panel(id) {
            ruang.lepas_semua_tombol_win32();

            if let Some(pengendali) = ruang.pengendali_arsip.take() {
                self.jadwalkan_penyelesaian_arsip(pengendali, None);
            }

            self.kepemilikan_tombol.batalkan_pemilik(id);

            drop(ruang);

            jendela.focus(&self.fokus, konteks);
            konteks.notify();
        }
    }

    pub(super) fn pilih_sesi(
        &mut self,
        id: IdTab,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.manajer.id_tab_aktif() == Some(id) || self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        self.ruang_mut().keadaan.seleksi.selesai();
        self.ruang_mut().arah_gulir_seleksi = 0;

        if self.manajer.pilih_tab(id) {
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

        if self.manajer.geser_tab(mundur) {
            jendela.focus(&self.fokus, konteks);
            konteks.notify();
        }
    }

    fn tutup_sesi(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        if let Some(id) = self.manajer.id_tab_aktif() {
            self.tutup_sesi_dengan_id(id, jendela, konteks);
        }
    }

    pub(super) fn tutup_sesi_dengan_id(
        &mut self,
        id: IdTab,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.manajer.jumlah_tab() <= 1 {
            self.ruang_mut().status = "Tab tidak dapat ditutup".into();

            konteks.notify();
            return;
        }

        let Some(susunan) = self.manajer.ambil_tab(id) else {
            return;
        };

        let daftar_id = susunan.daftar_id();

        let sesi_sibuk = daftar_id.iter().copied().any(|id_sesi| {
            self.manajer.ambil(id_sesi).is_some_and(|ruang| {
                ruang.sesi.is_some()
                    && (ruang.keadaan.siklus.berjalan()
                        || ruang.keadaan.motor.layar_alternatif_aktif())
            })
        });

        if sesi_sibuk {
            self.ruang_mut().status = "Hentikan perintah sebelum menutup tab".into();

            konteks.notify();
            return;
        }

        if daftar_id.iter().copied().any(|id_sesi| {
            self.manajer
                .ambil_mut(id_sesi)
                .is_some_and(RuangSesi::ada_output_arsip_tertunda)
        }) {
            self.ruang_mut().status =
                "Selesaikan sesi dengan exit dan tunggu pengarsipan sebelum menutup tab".to_owned();

            konteks.notify();
            return;
        }

        let mut pengendali_dilepas = Vec::new();

        for id_sesi in daftar_id.iter().copied() {
            if let Some(ruang) = self.manajer.ambil_mut(id_sesi) {
                ruang.lepas_semua_tombol_win32();

                if let Some(pengendali) = ruang.pengendali_arsip.take() {
                    pengendali_dilepas.push(pengendali);
                }
            }
        }

        drop(self.manajer.tutup_tab(id));

        for pengendali in pengendali_dilepas {
            self.jadwalkan_penyelesaian_arsip(pengendali, None);
        }

        for id_sesi in daftar_id {
            self.kepemilikan_tombol.batalkan_pemilik(id_sesi);
        }

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

            ("d", true) => {
                self.pecah_panel(ArahPanel::Horizontal, jendela, konteks);
            }

            ("e", true) => {
                self.pecah_panel(ArahPanel::Vertikal, jendela, konteks);
            }

            _ => return false,
        }

        true
    }
}
