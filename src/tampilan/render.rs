/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Rendering
 * Purpose: Renders ANSI output, selection and terminal application chrome
 */

use gpui::{
    Context, CursorStyle, MouseButton, ObjectFit, StyledImage, Window, div, img, prelude::*, px,
    rgb, rgba,
};

use waifujahat_terminal::terminal::IdSesi;

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};

const WARNA_SELEKSI: u32 = 0x493663;

impl RuangSesi {
    pub(super) fn baris_terminal(&self) -> Vec<String> {
        let mut baris = self.keadaan.layar.baris();

        if baris.is_empty() {
            baris.push(String::new());
        }

        let indeks_kursor = self
            .keadaan
            .layar
            .indeks_baris_kursor()
            .min(baris.len() - 1);

        let indeks_terakhir_berisi = baris
            .iter()
            .rposition(|teks| !teks.trim().is_empty())
            .unwrap_or(0);

        let indeks_akhir = indeks_kursor.max(indeks_terakhir_berisi);

        baris.truncate(indeks_akhir + 1);

        baris
    }

    pub(super) fn buat_keluaran_terminal(
        &self,
        id: IdSesi,
        konteks: &mut Context<JendelaTerminal>,
    ) -> impl IntoElement {
        let baris = self.baris_terminal();
        let jumlah_baris = baris.len();

        let baris_motor = self.keadaan.motor.baris_layar();
        let warna_motor = self.keadaan.motor.baris_berwarna(WARNA_TEKS, WARNA_LATAR);
        let pergeseran = jumlah_baris.saturating_sub(baris_motor.len());

        let mut keluaran = div()
            .id(format!("keluaran-terminal-{}", id.nilai()))
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.gulir)
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(move |terminal, kejadian, jendela, konteks| {
                    terminal.mouse_ditekan_panel(id, kejadian, jendela, konteks);
                    konteks.stop_propagation();
                }),
            );

        for (indeks, teks) in baris.into_iter().enumerate() {
            let tampilan = if teks.is_empty() {
                " ".to_string()
            } else {
                teks.clone()
            };

            let mut elemen_baris = div()
                .id(format!("baris-terminal-{indeks}"))
                .w_full()
                .h(px(18.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .whitespace_nowrap();

            if let Some((mulai, selesai)) = self.keadaan.seleksi.potongan_baris(indeks, &teks) {
                let karakter: Vec<char> = teks.chars().collect();

                let sebelum: String = karakter[..mulai].iter().collect();
                let dipilih: String = karakter[mulai..selesai].iter().collect();
                let sesudah: String = karakter[selesai..].iter().collect();

                elemen_baris = elemen_baris
                    .child(div().whitespace_nowrap().child(sebelum))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .bg(rgb(WARNA_SELEKSI))
                            .child(dipilih),
                    )
                    .child(div().whitespace_nowrap().child(sesudah));
            } else {
                let indeks_motor = indeks.checked_sub(pergeseran);

                let baris_cocok = indeks_motor
                    .and_then(|indeks| baris_motor.get(indeks).zip(warna_motor.get(indeks)))
                    .filter(|(teks_motor, _)| teks_motor.trim_end() == teks.trim_end());

                if let Some((_, potongan)) = baris_cocok {
                    let mut teks_berwarna = div().flex().items_center().whitespace_nowrap();

                    for bagian in potongan {
                        let mut elemen_teks =
                            div().whitespace_nowrap().text_color(rgb(bagian.warna_teks));

                        if bagian.warna_latar != WARNA_LATAR {
                            elemen_teks = elemen_teks.bg(rgb(bagian.warna_latar));
                        }

                        teks_berwarna = teks_berwarna.child(elemen_teks.child(bagian.teks.clone()));
                    }

                    elemen_baris = elemen_baris.child(teks_berwarna);
                } else {
                    elemen_baris = elemen_baris.child(div().whitespace_nowrap().child(tampilan));
                }
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}

impl Render for JendelaTerminal {
    fn render(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) -> impl IntoElement {
        let gambar_latar = konteks.global::<crate::aset::LatarAplikasi>().gambar();

        let halaman = div()
            .id("jendela-terminal")
            .relative()
            .overflow_hidden()
            .track_focus(&self.fokus)
            .on_key_down(konteks.listener(Self::proses_tombol))
            .on_key_up(konteks.listener(Self::proses_tombol_lepas))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, jendela, konteks| {
                    if terminal.tarikan_panel.is_some() {
                        return;
                    }

                    jendela.focus(&terminal.fokus, konteks);
                    terminal.ruang_mut().keadaan.seleksi.bersihkan();
                    konteks.notify();
                }),
            )
            .on_mouse_move(konteks.listener(|terminal, kejadian, jendela, konteks| {
                terminal.mouse_digerakkan(kejadian, jendela, konteks);
            }))
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|terminal, kejadian, jendela, konteks| {
                    terminal.tangani_lepas_mouse_tui(kejadian, jendela, konteks);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                konteks.listener(|terminal, kejadian, jendela, konteks| {
                    terminal.tangani_lepas_mouse_tui(kejadian, jendela, konteks);
                }),
            )
            .size_full()
            .flex()
            .flex_col()
            .p_2()
            .gap_1()
            .bg(rgb(WARNA_LATAR))
            .text_color(rgb(WARNA_TEKS))
            .font_family("Consolas")
            .text_size(px(14.0))
            .line_height(px(18.0))
            .child(
                img(gambar_latar)
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(ObjectFit::Cover),
            )
            .child(div().absolute().inset_0().bg(rgba(0x090d18b3)))
            .child(self.buat_tab_sesi(konteks));

        let mut halaman = halaman
            .child(self.buat_panel_terminal(jendela, konteks))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(WARNA_AKSEN))
                    .child(if self.ruang().keadaan.siklus.menunggu_konfirmasi() {
                        "Apakah input sudah benar?".to_string()
                    } else {
                        self.ruang().status.clone()
                    })
                    .child(if self.ruang().keadaan.siklus.menunggu_konfirmasi() {
                        "Tekan Y : Ya | N/Esc : Batal"
                    } else if self.ruang().keadaan.siklus.berjalan()
                        || self.ruang().keadaan.motor.layar_alternatif_aktif()
                    {
                        "Running - Esc untuk membatalkan"
                    } else if self.ruang().keadaan.siklus.konfirmasi_aktif() {
                        "Toggle Konfirmasi : ON (Tekan F8)"
                    } else {
                        "Toggle Konfirmasi : OFF (Tekan F8)"
                    }),
            );

        if let Some(posisi) = self.menu_split {
            halaman = halaman.child(self.buat_menu_split(posisi, jendela, konteks));
        }

        if let Some(lapisan) = self.buat_lapisan_tarikan_panel(konteks) {
            halaman = halaman.child(lapisan);
        }

        halaman
    }
}
