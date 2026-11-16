/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Alacritty grid rendering
 * Purpose: Renders native terminal cells, colors and text selection
 */

use gpui::{Context, CursorStyle, MouseButton, Window, div, prelude::*, px, rgb};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};
use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};
use waifujahat_terminal::terminal::IdSesi;

const WARNA_SELEKSI: u32 = 0x493663;

impl RuangSesi {
    pub(super) fn mode_grid_aktif(&self) -> bool {
        match std::env::var("WAIFUJAHAT_RENDER_GRID") {
            Ok(nilai) => nilai == "1",
            Err(_) => true,
        }
    }

    pub(super) fn baris_seleksi(&self) -> Vec<String> {
        if !self.mode_grid_aktif() {
            return self.baris_terminal();
        }

        self.keadaan.motor.semua_baris()
    }

    pub(super) fn buat_keluaran_grid(
        &self,
        id: IdSesi,
        panel_aktif: bool,
        jendela: &mut Window,
        konteks: &mut Context<JendelaTerminal>,
    ) -> impl IntoElement {
        let lebar_karakter = Self::lebar_karakter_grid(jendela);
        let snapshot = self.keadaan.motor.ambil_snapshot(WARNA_TEKS, WARNA_LATAR);
        let (baris_kursor, kolom_kursor) = snapshot.posisi_kursor;
        let kursor_terlihat = snapshot.kursor_terlihat;
        let jumlah_kolom = snapshot.kolom;
        let indeks_awal_tampilan = self.keadaan.motor.indeks_awal_tampilan();
        let sedang_melihat_riwayat = self.keadaan.motor.offset_riwayat() > 0;

        let mut keluaran = div()
            .id(format!("keluaran-grid-{}", id.nilai()))
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.gulir)
            .cursor(CursorStyle::IBeam)
            .on_scroll(konteks.listener(
                move |terminal, kejadian: &gpui::ScrollEvent, _, konteks| {
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

                    if let Some(ruang) = terminal.manajer.ambil_mut(id) {
                        ruang.keadaan.motor.geser_riwayat(jumlah);
                        ruang.keadaan.seleksi.bersihkan();
                    }

                    konteks.stop_propagation();
                    konteks.notify();
                },
            ))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(move |terminal, kejadian, jendela, konteks| {
                    terminal.mouse_ditekan_panel(id, kejadian, jendela, konteks);
                    konteks.stop_propagation();
                }),
            );

        for (indeks, baris) in snapshot.baris.into_iter().enumerate() {
            let kursor_aktif =
                panel_aktif && !sedang_melihat_riwayat && kursor_terlihat && indeks == baris_kursor;

            let indeks_seleksi = indeks_awal_tampilan + indeks;

            let rentang = self
                .keadaan
                .seleksi
                .potongan_baris(indeks_seleksi, &baris.teks);

            let mut elemen_baris = div()
                .id(format!("baris-grid-{indeks}"))
                .w_full()
                .h(px(18.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .whitespace_nowrap();

            let batas_kursor = if kursor_aktif {
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

                let lebar_sel = if sel.lebar_ganda {
                    lebar_karakter * 2.0
                } else {
                    lebar_karakter
                };

                let mut elemen_sel = div()
                    .w(px(lebar_sel))
                    .flex_shrink_0()
                    .whitespace_nowrap()
                    .text_color(rgb(sel.warna_teks));

                if terpilih {
                    elemen_sel = elemen_sel.bg(rgb(WARNA_SELEKSI));
                } else if !sel.latar_default {
                    elemen_sel = elemen_sel.bg(rgb(sel.warna_latar));
                }

                let mut elemen_sel = elemen_sel.child(sel.teks);

                if kursor_aktif && kolom == kolom_kursor {
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

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}
