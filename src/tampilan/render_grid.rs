/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Alacritty grid rendering
 * Purpose: Renders native terminal cells, colors and text selection
 */

use gpui::{Context, CursorStyle, MouseButton, div, prelude::*, px, rgb};

use super::JendelaTerminal;
use crate::tema::{WARNA_LATAR, WARNA_TEKS};

const WARNA_SELEKSI: u32 = 0x493663;

impl JendelaTerminal {
    pub(super) fn mode_grid_aktif(&self) -> bool {
        std::env::var("WAIFUJAHAT_RENDER_GRID").is_ok_and(|nilai| nilai == "1")
    }

    pub(super) fn baris_seleksi(&self) -> Vec<String> {
        if !self.mode_grid_aktif() {
            return self.baris_terminal();
        }

        let snapshot = self.motor.ambil_snapshot(WARNA_TEKS, WARNA_LATAR);
        let (indeks_kursor, _) = snapshot.posisi_kursor;

        let mut baris: Vec<String> = snapshot.baris.into_iter().map(|baris| baris.teks).collect();

        if !self.perintah_berjalan
            && let Some(teks) = baris.get_mut(indeks_kursor)
        {
            teks.push_str(self.masukan.teks());
        }
        baris
    }

    pub(super) fn buat_keluaran_grid(&self, konteks: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.motor.ambil_snapshot(WARNA_TEKS, WARNA_LATAR);
        let (baris_kursor, _) = snapshot.posisi_kursor;

        let mut keluaran = div()
            .id("keluaran-grid")
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

        for (indeks, baris) in snapshot.baris.into_iter().enumerate() {
            let editor_aktif = indeks == baris_kursor && !self.perintah_berjalan;

            let mut teks_seleksi = baris.teks.clone();

            if editor_aktif {
                teks_seleksi.push_str(self.masukan.teks());
            }

            let rentang = self.seleksi.potongan_baris(indeks, &teks_seleksi);

            let mut elemen_baris = div()
                .id(format!("baris-grid-{indeks}"))
                .w_full()
                .h(px(18.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .whitespace_nowrap();

            if editor_aktif {
                if let Some((mulai, selesai)) = rentang {
                    let karakter: Vec<char> = teks_seleksi.chars().collect();

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
                    elemen_baris = elemen_baris.child(self.masukan.tampilkan(&baris.teks));
                }
            } else {
                let batas = baris
                    .sel
                    .iter()
                    .rposition(|sel| {
                        !sel.latar_default || (sel.teks != " " && !sel.pengisi_lebar_ganda)
                    })
                    .map_or(0, |indeks| indeks + 1);

                let mut indeks_karakter = 0;

                for sel in baris.sel.into_iter().take(batas) {
                    if sel.pengisi_lebar_ganda {
                        continue;
                    }

                    let akhir_karakter = indeks_karakter + sel.teks.chars().count();

                    let terpilih = rentang.is_some_and(|(mulai, selesai)| {
                        indeks_karakter < selesai && akhir_karakter > mulai
                    });

                    indeks_karakter = akhir_karakter;

                    let mut elemen_sel = div().whitespace_nowrap().text_color(rgb(sel.warna_teks));

                    if terpilih {
                        elemen_sel = elemen_sel.bg(rgb(WARNA_SELEKSI));
                    } else if !sel.latar_default {
                        elemen_sel = elemen_sel.bg(rgb(sel.warna_latar));
                    }

                    elemen_baris = elemen_baris.child(elemen_sel.child(sel.teks));
                }
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}
