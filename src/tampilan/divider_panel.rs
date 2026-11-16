/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Pane divider
 * Purpose: Handles interactive resizing of terminal pane layouts
 */

use gpui::{
    AnyElement, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, ScrollHandle,
    div, prelude::*, px, rgb,
};

use waifujahat_terminal::terminal::{ArahPanel, CabangPanel, IdTab, SusunanPanel};

use super::JendelaTerminal;

use crate::tema::WARNA_AKSEN;

const UKURAN_DIVIDER: f32 = 6.0;
const WARNA_DIVIDER: u32 = 0x303649;

pub(super) struct TarikanPanel {
    tab: IdTab,
    jalur: Vec<CabangPanel>,
    arah: ArahPanel,
    rasio_awal: u16,
    posisi_awal: f32,
    panjang: f32,
}

impl JendelaTerminal {
    pub(super) fn buat_pemisah_panel(
        &self,
        jalur: &[CabangPanel],
        arah: ArahPanel,
        rasio: u16,
        batas: ScrollHandle,
        konteks: &mut Context<Self>,
    ) -> AnyElement {
        let tab = self
            .manajer
            .id_tab_aktif()
            .expect("Tab aktif tidak tersedia");
        let jalur = jalur.to_vec();

        let pemisah = div()
            .id(format!("divider-panel-{jalur:?}"))
            .flex_shrink_0()
            .rounded_sm()
            .bg(rgb(WARNA_DIVIDER))
            .hover(|gaya| gaya.bg(rgb(WARNA_AKSEN)));

        let pemisah = match arah {
            ArahPanel::Horizontal => pemisah
                .w(px(UKURAN_DIVIDER))
                .h_full()
                .cursor(CursorStyle::ResizeLeftRight),

            ArahPanel::Vertikal => pemisah
                .h(px(UKURAN_DIVIDER))
                .w_full()
                .cursor(CursorStyle::ResizeUpDown),
        };

        pemisah
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(move |terminal, kejadian: &MouseDownEvent, _, konteks| {
                    konteks.stop_propagation();

                    if terminal.ruang().mouse_tui_ditekan.is_some()
                        || terminal.manajer.id_tab_aktif() != Some(tab)
                    {
                        return;
                    }

                    let ukuran = batas.bounds().size;

                    let (posisi_awal, panjang) = match arah {
                        ArahPanel::Horizontal => {
                            (f32::from(kejadian.position.x), f32::from(ukuran.width))
                        }

                        ArahPanel::Vertikal => {
                            (f32::from(kejadian.position.y), f32::from(ukuran.height))
                        }
                    };

                    let panjang = panjang - UKURAN_DIVIDER;

                    if panjang <= 0.0 {
                        return;
                    }

                    terminal.ruang_mut().keadaan.seleksi.selesai();
                    terminal.ruang_mut().arah_gulir_seleksi = 0;

                    terminal.tarikan_panel = Some(TarikanPanel {
                        tab,
                        jalur: jalur.clone(),
                        arah,
                        rasio_awal: rasio,
                        posisi_awal,
                        panjang,
                    });

                    konteks.notify();
                }),
            )
            .into_any_element()
    }

    fn proses_tarikan_panel(&mut self, kejadian: &MouseMoveEvent, konteks: &mut Context<Self>) {
        if kejadian.pressed_button != Some(MouseButton::Left) {
            self.akhiri_tarikan_panel(konteks);
            return;
        }

        let Some(tarikan) = &self.tarikan_panel else {
            return;
        };

        if self.manajer.id_tab_aktif() != Some(tarikan.tab) {
            self.akhiri_tarikan_panel(konteks);
            return;
        }

        let posisi = match tarikan.arah {
            ArahPanel::Horizontal => f32::from(kejadian.position.x),
            ArahPanel::Vertikal => f32::from(kejadian.position.y),
        };

        let pergeseran = posisi - tarikan.posisi_awal;

        let Some(rasio) =
            SusunanPanel::hitung_rasio_tarikan(tarikan.rasio_awal, pergeseran, tarikan.panjang)
        else {
            self.akhiri_tarikan_panel(konteks);
            return;
        };

        if self
            .manajer
            .atur_rasio_panel(tarikan.tab, &tarikan.jalur, rasio)
        {
            konteks.notify();
        }
    }

    pub(super) fn akhiri_tarikan_panel(&mut self, konteks: &mut Context<Self>) {
        if self.tarikan_panel.take().is_some() {
            konteks.notify();
        }
    }

    pub(super) fn buat_lapisan_tarikan_panel(
        &self,
        konteks: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let tarikan = self.tarikan_panel.as_ref()?;

        let kursor = match tarikan.arah {
            ArahPanel::Horizontal => CursorStyle::ResizeLeftRight,
            ArahPanel::Vertikal => CursorStyle::ResizeUpDown,
        };

        Some(
            div()
                .id("lapisan-tarikan-panel")
                .absolute()
                .inset_0()
                .occlude()
                .cursor(kursor)
                .on_mouse_move(konteks.listener(|terminal, kejadian, _, konteks| {
                    terminal.proses_tarikan_panel(kejadian, konteks);
                    konteks.stop_propagation();
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    konteks.listener(|terminal, _, _, konteks| {
                        terminal.akhiri_tarikan_panel(konteks);
                        konteks.stop_propagation();
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    konteks.listener(|terminal, _, _, konteks| {
                        terminal.akhiri_tarikan_panel(konteks);
                        konteks.stop_propagation();
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    konteks.listener(|terminal, _, _, konteks| {
                        terminal.akhiri_tarikan_panel(konteks);
                    }),
                )
                .into_any_element(),
        )
    }
}
