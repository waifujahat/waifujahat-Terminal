/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Terminal search
 * Purpose: Manages pane-local search input and navigation
 */

use gpui::{Context, CursorStyle, KeyDownEvent, MouseButton, Window, div, prelude::*, px, rgb};

use waifujahat_terminal::terminal::{CakupanPencarian, IdSesi, OpsiPencarian, PencarianTerminal};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};

const WARNA_BILAH: u32 = 0x191e30;
const WARNA_HOVER: u32 = 0x303649;
const BATAS_KATA: usize = 512;

#[derive(Default)]
pub(super) struct PencarianPanel {
    pub(super) terbuka: bool,
    pub(super) lanjutan: bool,
    pub(super) kata: String,
    pub(super) hasil: PencarianTerminal,
    pub(super) cakupan: CakupanPencarian,
    pub(super) opsi: OpsiPencarian,
}

#[derive(Clone, Copy)]
enum AksiPencarian {
    Sebelumnya,
    Berikutnya,
    Lanjutan,
    Cakupan(CakupanPencarian),
    PekaHuruf,
    KataUtuh,
    Regex,
}

impl RuangSesi {
    pub(super) fn perbarui_pencarian(&mut self, kata_berubah: bool) {
        if !self.pencarian.terbuka {
            return;
        }

        if !self.mode_grid_aktif() || self.keadaan.motor.layar_alternatif_aktif() {
            self.pencarian = PencarianPanel::default();

            return;
        }

        if self.pencarian.kata.is_empty() {
            self.pencarian.hasil.bersihkan();

            return;
        }

        let baris = self.keadaan.motor.semua_baris();
        let jumlah_riwayat = self.keadaan.motor.jumlah_baris_riwayat();
        let cakupan = self.pencarian.cakupan;
        let opsi = self.pencarian.opsi;

        if kata_berubah {
            self.pencarian.hasil.cari_dengan_opsi(
                &baris,
                &self.pencarian.kata,
                jumlah_riwayat,
                cakupan,
                opsi,
            );

            self.tuju_hasil_pencarian();
        } else {
            self.pencarian.hasil.perbarui_dengan_opsi(
                &baris,
                &self.pencarian.kata,
                jumlah_riwayat,
                cakupan,
                opsi,
            );
        }
    }

    fn tuju_hasil_pencarian(&mut self) {
        if let Some(hasil) = self.pencarian.hasil.aktif() {
            self.keadaan.motor.tampilkan_baris(hasil.baris);

            self.keadaan.seleksi.bersihkan();
            self.arah_gulir_seleksi = 0;
        }
    }

    fn navigasi_pencarian(&mut self, mundur: bool) {
        if mundur {
            self.pencarian.hasil.sebelumnya();
        } else {
            self.pencarian.hasil.berikutnya();
        }

        self.tuju_hasil_pencarian();
    }

    fn tambah_kata_pencarian(&mut self, teks: &str) {
        let sisa = BATAS_KATA.saturating_sub(self.pencarian.kata.chars().count());

        let panjang_sebelum = self.pencarian.kata.len();

        self.pencarian.kata.extend(
            teks.chars()
                .filter(|karakter| !karakter.is_control())
                .take(sisa),
        );

        if self.pencarian.kata.len() != panjang_sebelum {
            self.perbarui_pencarian(true);
        }
    }
}

impl JendelaTerminal {
    pub(super) fn tangani_tombol_pencarian(
        &mut self,
        kejadian: &KeyDownEvent,
        konteks: &mut Context<Self>,
    ) -> bool {
        let tombol = &kejadian.keystroke;
        let nama = tombol.key.to_ascii_lowercase();

        let buka = nama == "f"
            && tombol.modifiers.control
            && tombol.modifiers.shift
            && !tombol.modifiers.alt
            && !tombol.modifiers.platform;

        if buka {
            self.buka_pencarian(konteks);

            return true;
        }

        if !self.ruang().pencarian.terbuka {
            return false;
        }

        if nama == "v" && tombol.modifiers.control {
            if let Some(teks) = konteks.read_from_clipboard().and_then(|isi| isi.text()) {
                self.ruang_mut().tambah_kata_pencarian(&teks);
            }

            konteks.notify();

            return true;
        }

        let ruang = self.ruang_mut();

        match nama.as_str() {
            "escape" | "esc" => {
                ruang.pencarian = PencarianPanel::default();
            }

            "enter" => {
                ruang.navigasi_pencarian(tombol.modifiers.shift);
            }

            "backspace" => {
                if ruang.pencarian.kata.pop().is_some() {
                    ruang.perbarui_pencarian(true);
                }
            }

            _ => {
                if !tombol.modifiers.control
                    && !tombol.modifiers.alt
                    && !tombol.modifiers.platform
                    && let Some(teks) = tombol.key_char.as_deref()
                {
                    ruang.tambah_kata_pencarian(teks);
                }
            }
        }

        konteks.notify();

        true
    }

    fn fokus_panel_pencarian(
        &mut self,
        id: IdSesi,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> bool {
        if self.tarikan_panel.is_some() {
            return false;
        }

        if self.manajer.id_panel_aktif() != Some(id) {
            if self.ruang().mouse_tui_ditekan.is_some() {
                return false;
            }

            self.ruang_mut().keadaan.seleksi.selesai();
            self.ruang_mut().arah_gulir_seleksi = 0;

            if !self.manajer.pilih_panel(id) {
                return false;
            }
        }

        jendela.focus(&self.fokus, konteks);

        true
    }

    fn buat_tombol_pencarian(
        id: IdSesi,
        label: &'static str,
        aksi: AksiPencarian,
        aktif: bool,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(format!("pencarian-{label}-{}", id.nilai()))
            .h(px(24.0))
            .px_2()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .bg(rgb(if aktif { WARNA_HOVER } else { WARNA_BILAH }))
            .cursor(CursorStyle::PointingHand)
            .text_color(rgb(WARNA_AKSEN))
            .hover(|gaya| gaya.bg(rgb(WARNA_HOVER)))
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(move |terminal, _, jendela, konteks| {
                    konteks.stop_propagation();

                    if !terminal.fokus_panel_pencarian(id, jendela, konteks) {
                        return;
                    }

                    let ruang = terminal.ruang_mut();

                    match aksi {
                        AksiPencarian::Sebelumnya => {
                            ruang.navigasi_pencarian(true);
                        }

                        AksiPencarian::Berikutnya => {
                            ruang.navigasi_pencarian(false);
                        }

                        AksiPencarian::Lanjutan => {
                            ruang.pencarian.lanjutan = !ruang.pencarian.lanjutan;
                        }

                        AksiPencarian::Cakupan(cakupan) => {
                            if ruang.pencarian.cakupan != cakupan {
                                ruang.pencarian.cakupan = cakupan;

                                ruang.perbarui_pencarian(true);
                            }
                        }

                        AksiPencarian::PekaHuruf => {
                            ruang.pencarian.opsi.peka_huruf = !ruang.pencarian.opsi.peka_huruf;

                            ruang.perbarui_pencarian(true);
                        }

                        AksiPencarian::KataUtuh => {
                            ruang.pencarian.opsi.kata_utuh = !ruang.pencarian.opsi.kata_utuh;

                            ruang.perbarui_pencarian(true);
                        }

                        AksiPencarian::Regex => {
                            ruang.pencarian.opsi.regex = !ruang.pencarian.opsi.regex;

                            ruang.perbarui_pencarian(true);
                        }
                    }

                    konteks.notify();
                }),
            )
    }

    pub(super) fn buka_pencarian(&mut self, konteks: &mut Context<Self>) {
        if self.tarikan_panel.is_some() {
            return;
        }

        let ruang = self.ruang_mut();

        if !ruang.mode_grid_aktif()
            || ruang.keadaan.motor.layar_alternatif_aktif()
            || ruang.mouse_tui_ditekan.is_some()
        {
            return;
        }

        ruang.keadaan.seleksi.bersihkan();
        ruang.arah_gulir_seleksi = 0;
        ruang.pencarian.terbuka = true;

        self.menu_split = None;

        konteks.notify();
    }

    pub(super) fn ubah_pencarian(&mut self, konteks: &mut Context<Self>) {
        if self.tarikan_panel.is_some() {
            return;
        }

        if self.ruang().pencarian.terbuka {
            self.ruang_mut().pencarian = PencarianPanel::default();

            konteks.notify();
        } else {
            self.buka_pencarian(konteks);
        }
    }

    pub(super) fn buat_bilah_pencarian(
        &self,
        id: IdSesi,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let ruang = self.manajer.ambil(id).expect("Sesi panel tidak tersedia");

        let jumlah = ruang.pencarian.hasil.jumlah();

        let aktif = ruang.pencarian.hasil.nomor_aktif().unwrap_or(0);

        let lanjutan = ruang.pencarian.lanjutan;
        let cakupan = ruang.pencarian.cakupan;
        let opsi = ruang.pencarian.opsi;

        let regex_tidak_valid = ruang.pencarian.hasil.kesalahan().is_some();

        let status_hasil = if regex_tidak_valid {
            "Invalid regex".to_string()
        } else {
            format!("{aktif}/{jumlah}")
        };

        let kata = if ruang.pencarian.kata.is_empty() {
            "Search".to_string()
        } else {
            ruang.pencarian.kata.clone()
        };

        let baris_utama = div()
            .w_full()
            .min_w_0()
            .h(px(32.0))
            .flex()
            .items_center()
            .gap_1()
            .child(Self::buat_tombol_pencarian(
                id,
                if lanjutan { "⌄" } else { ">" },
                AksiPencarian::Lanjutan,
                lanjutan || cakupan != CakupanPencarian::Semua,
                konteks,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .px_2()
                    .py_1()
                    .bg(rgb(WARNA_LATAR))
                    .text_color(rgb(WARNA_TEKS))
                    .cursor(CursorStyle::IBeam)
                    .child(kata),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(rgb(WARNA_AKSEN))
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(rgb(if regex_tidak_valid {
                                0xf87171
                            } else {
                                WARNA_AKSEN
                            }))
                            .child(status_hasil),
                    ),
            )
            .child(Self::buat_tombol_pencarian(
                id,
                "↑",
                AksiPencarian::Sebelumnya,
                false,
                konteks,
            ))
            .child(Self::buat_tombol_pencarian(
                id,
                "↓",
                AksiPencarian::Berikutnya,
                false,
                konteks,
            ));

        let mut bilah = div()
            .id(format!("bilah-pencarian-{}", id.nilai()))
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .px_1()
            .bg(rgb(WARNA_BILAH))
            .rounded_sm()
            .child(baris_utama);

        if lanjutan {
            bilah = bilah
                .child(
                    div()
                        .w_full()
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(Self::buat_tombol_pencarian(
                            id,
                            "All",
                            AksiPencarian::Cakupan(CakupanPencarian::Semua),
                            cakupan == CakupanPencarian::Semua,
                            konteks,
                        ))
                        .child(Self::buat_tombol_pencarian(
                            id,
                            "Screen",
                            AksiPencarian::Cakupan(CakupanPencarian::LayarAktif),
                            cakupan == CakupanPencarian::LayarAktif,
                            konteks,
                        ))
                        .child(Self::buat_tombol_pencarian(
                            id,
                            "History",
                            AksiPencarian::Cakupan(CakupanPencarian::Riwayat),
                            cakupan == CakupanPencarian::Riwayat,
                            konteks,
                        )),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(Self::buat_tombol_pencarian(
                            id,
                            "Aa",
                            AksiPencarian::PekaHuruf,
                            opsi.peka_huruf,
                            konteks,
                        ))
                        .child(Self::buat_tombol_pencarian(
                            id,
                            "ab",
                            AksiPencarian::KataUtuh,
                            opsi.kata_utuh,
                            konteks,
                        ))
                        .child(Self::buat_tombol_pencarian(
                            id,
                            ".*",
                            AksiPencarian::Regex,
                            opsi.regex,
                            konteks,
                        )),
                );
        }

        bilah.on_mouse_down(
            MouseButton::Left,
            konteks.listener(move |terminal, _, jendela, konteks| {
                konteks.stop_propagation();

                if terminal.fokus_panel_pencarian(id, jendela, konteks) {
                    konteks.notify();
                }
            }),
        )
    }
}
