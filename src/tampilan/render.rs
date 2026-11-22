/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Rendering
 * Purpose : Renders ANSI output, selection and terminal application chrome
 */

use gpui::{
    Context, CursorStyle, MouseButton, ObjectFit, StyledImage, Window, div, img, prelude::*, px,
    rgb, rgba,
};

use waifujahat_terminal::terminal::IdSesi;

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

use crate::tema::kontras::{PaletKontras, WARNA_LAPISAN};
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

        let mut palet = PaletKontras::baru(self.pengaturan.alfa_latar());

        let mut buat_teks =
            |teks: String, warna_teks: u32, warna_latar: Option<u32>, tersembunyi: bool| {
                let gaya = palet.gaya(&teks, warna_teks, warna_latar, tersembunyi);

                let mut elemen = div()
                    .whitespace_nowrap()
                    .text_color(gaya.warna_teks.map_or(rgba(0), rgb));

                if let Some(latar) = gaya.warna_latar {
                    elemen = elemen.bg(rgb(latar));
                }

                elemen.child(teks)
            };

        let mut keluaran = div()
            .id(format!("keluaran-terminal-{}", id.nilai()))
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.gulir)
            .font_family(self.pengaturan.keluarga_font.clone())
            .text_size(px(self.pengaturan.ukuran_font_efektif()))
            .line_height(px(self.pengaturan.tinggi_baris()))
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
                .h(px(self.pengaturan.tinggi_baris()))
                .flex_shrink_0()
                .flex()
                .items_center()
                .whitespace_nowrap();

            let rentang = self.keadaan.seleksi.potongan_baris(indeks, &teks);

            let indeks_motor = indeks.checked_sub(pergeseran);

            let baris_cocok = indeks_motor
                .and_then(|indeks| baris_motor.get(indeks).zip(warna_motor.get(indeks)))
                .filter(|(teks_motor, _)| teks_motor.trim_end() == teks.trim_end());

            let mut buat_potongan = |teks: &str,
                                     posisi: usize,
                                     warna_teks: u32,
                                     warna_latar: Option<u32>,
                                     tersembunyi: bool| {
                let panjang = teks.chars().count();

                let (mulai, selesai) = rentang.map_or((0, 0), |(mulai, selesai)| {
                    (
                        mulai.saturating_sub(posisi).min(panjang),
                        selesai.saturating_sub(posisi).min(panjang),
                    )
                });

                let mut elemen = div().flex().items_center().whitespace_nowrap();

                if mulai == selesai {
                    return elemen.child(buat_teks(
                        teks.to_owned(),
                        warna_teks,
                        warna_latar,
                        tersembunyi,
                    ));
                }

                let karakter: Vec<char> = teks.chars().collect();

                for (potongan, latar) in [
                    (&karakter[..mulai], warna_latar),
                    (&karakter[mulai..selesai], Some(WARNA_SELEKSI)),
                    (&karakter[selesai..], warna_latar),
                ] {
                    if potongan.is_empty() {
                        continue;
                    }

                    elemen = elemen.child(buat_teks(
                        potongan.iter().collect(),
                        warna_teks,
                        latar,
                        tersembunyi,
                    ));
                }

                elemen
            };

            if let Some((_, potongan)) = baris_cocok {
                let mut teks_berwarna = div().flex().items_center().whitespace_nowrap();

                let mut posisi = 0;

                for bagian in potongan {
                    let latar = (!bagian.latar_default).then_some(bagian.warna_latar);

                    teks_berwarna = teks_berwarna.child(buat_potongan(
                        &bagian.teks,
                        posisi,
                        bagian.warna_teks,
                        latar,
                        bagian.tersembunyi,
                    ));

                    posisi += bagian.teks.chars().count();
                }

                if rentang.is_some() {
                    let sisa: String = teks.chars().skip(posisi).collect();

                    if !sisa.is_empty() {
                        teks_berwarna = teks_berwarna
                            .child(buat_potongan(&sisa, posisi, WARNA_TEKS, None, false));
                    }
                }

                elemen_baris = elemen_baris.child(teks_berwarna);
            } else {
                elemen_baris =
                    elemen_baris.child(buat_potongan(&tampilan, 0, WARNA_TEKS, None, false));
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}

impl Render for JendelaTerminal {
    fn render(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) -> impl IntoElement {
        let gambar_latar = konteks.global::<crate::aset::LatarAplikasi>().gambar();

        let warna_lapisan = (WARNA_LAPISAN << 8) | u32::from(self.pengaturan.alfa_latar());

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
            .child(div().absolute().inset_0().bg(rgba(warna_lapisan)))
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
                    .child(if self.ruang().sesi.is_none() {
                        "Sesi berakhir"
                    } else if self.ruang().profil_lokal().is_none() {
                        "SSH session"
                    } else if !self.ruang().mendukung_integrasi_shell() {
                        "Direct shell input"
                    } else if self.ruang().keadaan.siklus.menunggu_konfirmasi() {
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

        if let Some(menu) = &self.menu_profil {
            halaman = halaman.child(self.buat_menu_profil(menu, jendela, konteks));
        }

        if self.edit_nama_tab.is_some() {
            halaman = halaman.child(self.buat_edit_nama_tab(jendela, konteks));
        }

        if let Some(lapisan) = self.buat_lapisan_tarikan_panel(konteks) {
            halaman = halaman.child(lapisan);
        }

        halaman
    }
}
