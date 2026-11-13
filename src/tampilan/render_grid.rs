/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Alacritty grid rendering
 * Purpose: Renders native terminal cells, colors and text selection
 */

use gpui::{Context, CursorStyle, MouseButton, div, prelude::*, px, rgb};

use super::JendelaTerminal;
use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};

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
            && self.motor.offset_riwayat() == 0
            && let Some(teks) = baris.get_mut(indeks_kursor)
        {
            teks.push_str(self.masukan.teks());
        }
        baris
    }

    pub(super) fn buat_keluaran_grid(&self, konteks: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.motor.ambil_snapshot(WARNA_TEKS, WARNA_LATAR);
        let (baris_kursor, kolom_kursor) = snapshot.posisi_kursor;
        let kursor_terlihat = snapshot.kursor_terlihat;
        let jumlah_kolom = snapshot.kolom;
        let sedang_melihat_riwayat = self.motor.offset_riwayat() > 0;

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
            .on_scroll(
                konteks.listener(|terminal, kejadian: &gpui::ScrollEvent, _, konteks| {
                    let delta = kejadian.delta.pixel_delta(px(18.0));
                    let jarak = f32::from(delta.y);

                    if jarak == 0.0 {
                        return;
                    }

                    let jumlah = (jarak / 18.0).round() as isize;

                    let jumlah = if jumlah == 0 {
                        jarak.signum() as isize
                    } else {
                        jumlah
                    };

                    terminal.motor.geser_riwayat(jumlah);
                    terminal.seleksi.bersihkan();

                    konteks.stop_propagation();
                    konteks.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, kejadian, jendela, konteks| {
                    terminal.mouse_ditekan(kejadian, jendela, konteks);
                    konteks.stop_propagation();
                }),
            );

        for (indeks, baris) in snapshot.baris.into_iter().enumerate() {
            let editor_aktif =
                !sedang_melihat_riwayat && indeks == baris_kursor && !self.perintah_berjalan;

            let kursor_program_aktif = !sedang_melihat_riwayat
                && self.perintah_berjalan
                && kursor_terlihat
                && indeks == baris_kursor;

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
                    let (sebelum, sesudah) = self.masukan.bagian_kursor();

                    let teks_sebelum = format!("{}{}", baris.teks, sebelum);

                    let kursor = div()
                        .relative()
                        .w(px(0.0))
                        .h(px(18.0))
                        .flex_shrink_0()
                        .child(
                            div()
                                .absolute()
                                .left(px(0.0))
                                .top(px(1.0))
                                .w(px(2.0))
                                .h(px(16.0))
                                .bg(rgb(WARNA_AKSEN)),
                        );

                    elemen_baris = elemen_baris
                        .child(div().whitespace_nowrap().child(teks_sebelum))
                        .child(kursor)
                        .child(div().whitespace_nowrap().child(sesudah.to_string()));
                }
            } else {
                let batas_kursor = if kursor_program_aktif {
                    kolom_kursor.saturating_add(1)
                } else {
                    0
                };

                let batas = baris
                    .sel
                    .iter()
                    .rposition(|sel| {
                        !sel.latar_default || (sel.teks != " " && !sel.pengisi_lebar_ganda)
                    })
                    .map_or(0, |indeks| indeks + 1)
                    .max(batas_kursor)
                    .min(jumlah_kolom);

                let mut indeks_karakter = 0;

                for (kolom, sel) in baris.sel.into_iter().take(batas).enumerate() {
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

                    let mut elemen_sel = elemen_sel.child(sel.teks);

                    if kursor_program_aktif && kolom == kolom_kursor {
                        elemen_sel = elemen_sel.relative().child(
                            div()
                                .absolute()
                                .left(px(0.0))
                                .top(px(1.0))
                                .w(px(2.0))
                                .h(px(16.0))
                                .bg(rgb(WARNA_AKSEN)),
                        );
                    }

                    elemen_baris = elemen_baris.child(elemen_sel);
                }
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}
