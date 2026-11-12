/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Rendering
 * Purpose: Renders ANSI output, selection, and terminal application chrome.
 */

use gpui::{Context, CursorStyle, MouseButton, Window, div, prelude::*, px, rgb};

use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};
use super::JendelaTerminal;

const WARNA_SELEKSI: u32 = 0x493663;

impl JendelaTerminal {
    pub(super) fn baris_terminal(&self) -> Vec<String> {
        let mut baris = self.layar.baris();

        while baris.last().is_some_and(|teks| teks.is_empty()) {
            baris.pop();
        }

        let baris_terakhir = baris.pop().unwrap_or_default();

        baris.push(format!("{baris_terakhir}{}", self.masukan.teks()));
        baris
    }

    fn buat_informasi_terminal(&self) -> impl IntoElement {
        div()
            .text_color(rgb(WARNA_TEKS))
            .child("Waifujahat Terminal 0.1.0")
    }

    fn buat_keluaran_terminal(&self, konteks: &mut Context<Self>) -> impl IntoElement {
        let baris = self.baris_terminal();
        let jumlah_baris = baris.len();

        let baris_motor = self.motor.baris_layar();
        let warna_motor = self.motor.baris_berwarna(WARNA_TEKS, WARNA_LATAR);
        let pergeseran = jumlah_baris.saturating_sub(baris_motor.len());

        let awalan = self
            .layar
            .baris()
            .into_iter()
            .rev()
            .find(|teks| !teks.is_empty())
            .unwrap_or_default();

        let mut keluaran = div()
            .id("keluaran-terminal")
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
                konteks.listener(|terminal, kejadian, jendela, konteks| {
                    terminal.mouse_ditekan(kejadian, jendela, konteks);
                    konteks.stop_propagation();
                }),
            );

        for (indeks, teks) in baris.into_iter().enumerate() {
            let terakhir = indeks + 1 == jumlah_baris;

            let tampilan = if terakhir {
                self.masukan.tampilkan(&awalan)
            } else if teks.is_empty() {
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

            if let Some((mulai, selesai)) = self.seleksi.potongan_baris(indeks, &teks) {
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
            } else if !terakhir {
                let indeks_motor = indeks.checked_sub(pergeseran);

                let baris_cocok = indeks_motor
                    .and_then(|indeks| baris_motor.get(indeks).zip(warna_motor.get(indeks)))
                    .filter(|(teks_motor, _)| teks_motor.trim_end() == teks.trim_end());

                if let Some((_, potongan)) = baris_cocok {
                    let mut teks_berwarna = div().flex().items_center().whitespace_nowrap();

                    for bagian in potongan {
                        teks_berwarna = teks_berwarna.child(
                            div()
                                .whitespace_nowrap()
                                .text_color(rgb(bagian.warna_teks))
                                .bg(rgb(bagian.warna_latar))
                                .child(bagian.teks.clone()),
                        );
                    }

                    elemen_baris = elemen_baris.child(teks_berwarna);
                } else {
                    elemen_baris = elemen_baris.child(div().whitespace_nowrap().child(tampilan));
                }
            } else {
                elemen_baris = elemen_baris.child(div().whitespace_nowrap().child(tampilan));
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}

impl Render for JendelaTerminal {
    fn render(&mut self, _jendela: &mut Window, konteks: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("jendela-terminal")
            .track_focus(&self.fokus)
            .on_key_down(konteks.listener(Self::proses_tombol))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, jendela, konteks| {
                    jendela.focus(&terminal.fokus, konteks);
                    terminal.seleksi.bersihkan();
                    konteks.notify();
                }),
            )
            .on_mouse_move(konteks.listener(|terminal, kejadian, jendela, konteks| {
                terminal.mouse_digerakkan(kejadian, jendela, konteks);
            }))
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, _| {
                    terminal.seleksi.selesai();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, _| {
                    terminal.seleksi.selesai();
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
            .child(self.buat_informasi_terminal())
            .child(self.buat_keluaran_terminal(konteks))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(WARNA_AKSEN))
                    .child(self.status.clone()),
            )
    }
}
