/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Alacritty grid rendering
 * Purpose: Renders native terminal cells, colors and text selection
 */

use gpui::{Context, CursorStyle, MouseButton, Window, div, prelude::*, px, rgb};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};
use crate::tema::kontras::PaletKontras;
use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};
use waifujahat_terminal::terminal::IdSesi;

const WARNA_SELEKSI: u32 = 0x493663;
const WARNA_PENCARIAN: u32 = 0x65502b;
const WARNA_PENCARIAN_AKTIF: u32 = 0xd49b35;

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
        let lebar_karakter = self.lebar_karakter_grid(jendela);
        let tinggi_baris = self.pengaturan.tinggi_baris();
        let mut palet = PaletKontras::baru(self.pengaturan.alfa_latar());
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
            .overflow_hidden()
            .track_scroll(&self.gulir)
            .font_family(self.pengaturan.keluarga_font.clone())
            .text_size(px(self.pengaturan.ukuran_font_efektif()))
            .line_height(px(tinggi_baris))
            .cursor(CursorStyle::IBeam)
            .on_scroll(konteks.listener(
                move |terminal, kejadian: &gpui::ScrollEvent, jendela, konteks| {
                    let delta = kejadian.delta.pixel_delta(px(tinggi_baris));

                    let jarak = f32::from(delta.y);

                    if let Some(ruang) = terminal.manajer.ambil_mut(id) {
                        let jumlah = ruang.pengaturan.langkah_gulir_grid(jarak);

                        if jumlah == 0 {
                            return;
                        }

                        ruang.keadaan.motor.geser_riwayat(jumlah);

                        ruang.arah_gulir_seleksi = 0;

                        if ruang.keadaan.seleksi.sedang_memilih()
                            && let Some(posisi) =
                                ruang.posisi_dari_mouse(kejadian.position, jendela)
                        {
                            ruang.keadaan.seleksi.perbarui(posisi);
                        }
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

            let hasil_pencarian = self.pencarian.hasil.hasil_baris(indeks_seleksi);

            let hasil_aktif = self
                .pencarian
                .hasil
                .aktif()
                .filter(|hasil| hasil.baris == indeks_seleksi);

            let rentang = self
                .keadaan
                .seleksi
                .potongan_baris(indeks_seleksi, &baris.teks);

            let mut elemen_baris = div()
                .id(format!("baris-grid-{indeks}"))
                .w_full()
                .h(px(tinggi_baris))
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

                let indeks_hasil =
                    hasil_pencarian.partition_point(|hasil| hasil.selesai <= indeks_karakter);

                let cocok = hasil_pencarian
                    .get(indeks_hasil)
                    .is_some_and(|hasil| hasil.mulai < akhir_karakter);

                let cocok_aktif = hasil_aktif.is_some_and(|hasil| {
                    indeks_karakter < hasil.selesai && akhir_karakter > hasil.mulai
                });

                indeks_karakter = akhir_karakter;

                let lebar_sel = if sel.lebar_ganda {
                    lebar_karakter * 2.0
                } else {
                    lebar_karakter
                };

                let (warna_teks, warna_latar) = if terpilih {
                    (sel.warna_teks, Some(WARNA_SELEKSI))
                } else if cocok_aktif {
                    (0x101010, Some(WARNA_PENCARIAN_AKTIF))
                } else if cocok {
                    (sel.warna_teks, Some(WARNA_PENCARIAN))
                } else if !sel.latar_default {
                    (sel.warna_teks, Some(sel.warna_latar))
                } else {
                    (sel.warna_teks, None)
                };

                let gaya = palet.gaya(&sel.teks, warna_teks, warna_latar, sel.tersembunyi);

                let mut elemen_sel = div()
                    .w(px(lebar_sel))
                    .flex_shrink_0()
                    .whitespace_nowrap()
                    .text_color(rgb(gaya.warna_teks.unwrap_or(WARNA_TEKS)));

                if let Some(latar) = gaya.warna_latar {
                    elemen_sel = elemen_sel.bg(rgb(latar));
                }

                let teks = if gaya.warna_teks.is_none() {
                    " ".to_owned()
                } else {
                    sel.teks
                };

                let mut elemen_sel = elemen_sel.child(teks);

                if kursor_aktif && kolom == kolom_kursor {
                    elemen_sel = elemen_sel.relative().child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(1.0))
                            .w(px(2.0))
                            .h(px(self.pengaturan.tinggi_kursor()))
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
