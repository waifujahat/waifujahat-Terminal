/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session tabs
 * Purpose: Renders interactive terminal session tabs
 */

use gpui::{Context, MouseButton, div, prelude::*, px, rgb};

use super::JendelaTerminal;

use crate::tema::{WARNA_AKSEN, WARNA_TEKS};

const WARNA_TAB_AKTIF: u32 = 0x493663;
const WARNA_TAB_NONAKTIF: u32 = 0x202536;

impl JendelaTerminal {
    pub(super) fn buat_tab_sesi(&self, konteks: &mut Context<Self>) -> impl IntoElement {
        let id_aktif = self.manajer.id_tab_aktif();

        let mut daftar = div()
            .id("daftar-tab-sesi")
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap_1()
            .overflow_x_scroll();

        for id in self.manajer.daftar_tab() {
            let aktif = id_aktif == Some(id);

            let warna = if aktif {
                WARNA_TAB_AKTIF
            } else {
                WARNA_TAB_NONAKTIF
            };

            let mut tab = div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .h(px(22.0))
                .bg(rgb(warna))
                .child(
                    div()
                        .id(format!("pilih-sesi-{}", id.nilai()))
                        .px_2()
                        .text_color(rgb(WARNA_TEKS))
                        .child(format!("waifujahat {}", id.nilai()))
                        .on_mouse_down(
                            MouseButton::Left,
                            konteks.listener(move |terminal, _, jendela, konteks| {
                                terminal.pilih_sesi(id, jendela, konteks);

                                konteks.stop_propagation();
                            }),
                        ),
                );

            if self.manajer.jumlah_tab() > 1 {
                tab = tab.child(
                    div()
                        .id(format!("tutup-sesi-{}", id.nilai()))
                        .px_2()
                        .text_color(rgb(WARNA_AKSEN))
                        .child("×")
                        .on_mouse_down(
                            MouseButton::Left,
                            konteks.listener(move |terminal, _, jendela, konteks| {
                                terminal.tutup_sesi_dengan_id(id, jendela, konteks);

                                konteks.stop_propagation();
                            }),
                        ),
                );
            }

            daftar = daftar.child(tab);
        }

        daftar = daftar.child(
            div()
                .id("tambah-sesi")
                .w(px(26.0))
                .h(px(22.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(WARNA_TAB_NONAKTIF))
                .text_color(rgb(WARNA_AKSEN))
                .child("+")
                .on_mouse_down(
                    MouseButton::Left,
                    konteks.listener(|terminal, _, jendela, konteks| {
                        terminal.tambah_sesi(jendela, konteks);

                        konteks.stop_propagation();
                    }),
                ),
        );

        div()
            .id("tab-terminal")
            .w_full()
            .h(px(22.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_1()
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|_, _, _, konteks| {
                    konteks.stop_propagation();
                }),
            )
            .child(daftar)
    }
}
