/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Pane rendering
 * Purpose: Renders recursive pane layouts, controls and resize dividers
 */

use gpui::{
    AnyElement, Context, CursorStyle, MouseButton, ScrollHandle, Window, div, prelude::*, px, rgb,
};

use waifujahat_terminal::terminal::{ArahPanel, CabangPanel, IdSesi, SimpulPanel};

use super::JendelaTerminal;

use crate::tema::WARNA_AKSEN;

const WARNA_BORDER_PANEL: u32 = 0x303649;
const WARNA_HOVER_TUTUP: u32 = 0x723b48;

impl JendelaTerminal {
    pub(super) fn buat_panel_terminal(
        &self,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let susunan = self.manajer.tab_aktif().expect("Tab aktif tidak tersedia");

        div()
            .id("susunan-panel")
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .child(self.buat_simpul_panel(
                susunan.akar(),
                &[],
                susunan.jumlah() > 1,
                jendela,
                konteks,
            ))
    }

    fn buat_kontrol_panel(&self, id: IdSesi, konteks: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .h(px(20.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_end()
            .child(
                div()
                    .id(format!("tutup-panel-{}", id.nilai()))
                    .w(px(20.0))
                    .h(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .cursor(CursorStyle::PointingHand)
                    .text_color(rgb(WARNA_AKSEN))
                    .hover(|gaya| gaya.bg(rgb(WARNA_HOVER_TUTUP)).text_color(rgb(0xffffff)))
                    .child("×")
                    .on_mouse_down(
                        MouseButton::Left,
                        konteks.listener(move |terminal, _, jendela, konteks| {
                            konteks.stop_propagation();

                            terminal.tutup_panel_dengan_id(id, jendela, konteks);
                        }),
                    ),
            )
            .into_any_element()
    }

    fn bungkus_panel(bobot: u16, isi: AnyElement) -> AnyElement {
        let mut pembungkus = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .child(isi);

        pembungkus.style().flex_grow = Some(f32::from(bobot));

        pembungkus.into_any_element()
    }

    fn buat_simpul_panel(
        &self,
        simpul: &SimpulPanel,
        jalur: &[CabangPanel],
        tampilkan_border: bool,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> AnyElement {
        match simpul {
            SimpulPanel::Tunggal(id) => {
                let ruang = self.manajer.ambil(*id).expect("Sesi panel tidak tersedia");

                let aktif = self.manajer.id_panel_aktif() == Some(*id);

                let mut panel = div()
                    .id(format!("panel-terminal-{}", id.nilai()))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden();

                if tampilkan_border {
                    panel = panel
                        .border_1()
                        .border_color(rgb(if aktif {
                            WARNA_AKSEN
                        } else {
                            WARNA_BORDER_PANEL
                        }))
                        .p_2()
                        .gap_1()
                        .child(self.buat_kontrol_panel(*id, konteks));
                }

                if ruang.pencarian.terbuka {
                    panel = panel.child(self.buat_bilah_pencarian(*id, konteks));
                }

                if ruang.mode_grid_aktif() {
                    panel = panel.child(ruang.buat_keluaran_grid(*id, aktif, jendela, konteks));
                } else {
                    panel = panel.child(ruang.buat_keluaran_terminal(*id, konteks));
                }

                panel.into_any_element()
            }

            SimpulPanel::Terbagi {
                arah,
                rasio,
                pertama,
                kedua,
            } => {
                let batas = ScrollHandle::new();

                let kelompok = div()
                    .id(format!("kelompok-panel-{jalur:?}"))
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .track_scroll(&batas);

                let kelompok = match arah {
                    ArahPanel::Horizontal => kelompok.flex_row(),
                    ArahPanel::Vertikal => kelompok.flex_col(),
                };

                let mut jalur_pertama = jalur.to_vec();
                jalur_pertama.push(CabangPanel::Pertama);

                let mut jalur_kedua = jalur.to_vec();
                jalur_kedua.push(CabangPanel::Kedua);

                let panel_pertama = self.buat_simpul_panel(
                    pertama,
                    &jalur_pertama,
                    tampilkan_border,
                    jendela,
                    konteks,
                );

                let panel_kedua =
                    self.buat_simpul_panel(kedua, &jalur_kedua, tampilkan_border, jendela, konteks);

                let pemisah = self.buat_pemisah_panel(jalur, *arah, *rasio, batas, konteks);

                kelompok
                    .child(Self::bungkus_panel(*rasio, panel_pertama))
                    .child(pemisah)
                    .child(Self::bungkus_panel(1000 - *rasio, panel_kedua))
                    .into_any_element()
            }
        }
    }
}
