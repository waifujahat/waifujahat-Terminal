/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Split pane menu
 * Purpose: Provides interactive pane splitting controls
 */

use gpui::{Context, CursorStyle, MouseButton, Pixels, Window, div, prelude::*, px, rgb};

use waifujahat_terminal::terminal::ArahPanel;

use super::JendelaTerminal;

use crate::tema::{WARNA_AKSEN, WARNA_TEKS};

const WARNA_MENU: u32 = 0x202536;
const WARNA_HOVER: u32 = 0x493663;
const WARNA_BORDER: u32 = 0x393f52;

impl JendelaTerminal {
    pub(super) fn buat_menu_split(
        &self,
        posisi: Pixels,
        jendela: &Window,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let lebar_jendela = f32::from(jendela.viewport_size().width);

        let lebar_menu = (lebar_jendela - 16.0).clamp(1.0, 340.0);

        let posisi_x = f32::from(posisi).clamp(0.0, (lebar_jendela - lebar_menu - 8.0).max(0.0));

        div()
            .id("lapisan-menu-split")
            .absolute()
            .inset_0()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, konteks| {
                    terminal.menu_split = None;

                    konteks.stop_propagation();
                    konteks.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|_, _, _, konteks| {
                    konteks.stop_propagation();
                }),
            )
            .child(
                div()
                    .id("menu-split")
                    .absolute()
                    .left(px(posisi_x))
                    .top(px(34.0))
                    .w(px(lebar_menu))
                    .flex()
                    .flex_col()
                    .p_1()
                    .gap_1()
                    .bg(rgb(WARNA_MENU))
                    .border_1()
                    .border_color(rgb(WARNA_BORDER))
                    .rounded_md()
                    .on_mouse_down(
                        MouseButton::Left,
                        konteks.listener(|_, _, _, konteks| {
                            konteks.stop_propagation();
                        }),
                    )
                    .child(self.buat_pilihan_split(
                        "split-horizontal",
                        "Split Horizontal",
                        "Ctrl+Shift+D",
                        ArahPanel::Horizontal,
                        konteks,
                    ))
                    .child(self.buat_pilihan_split(
                        "split-vertical",
                        "Split Vertical",
                        "Ctrl+Shift+E",
                        ArahPanel::Vertikal,
                        konteks,
                    )),
            )
    }

    fn buat_pilihan_split(
        &self,
        id: &'static str,
        nama: &'static str,
        shortcut: &'static str,
        arah: ArahPanel,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .w_full()
            .h(px(28.0))
            .px_2()
            .flex()
            .items_center()
            .justify_between()
            .rounded_sm()
            .cursor(CursorStyle::PointingHand)
            .hover(|gaya| gaya.bg(rgb(WARNA_HOVER)))
            .child(div().text_color(rgb(WARNA_TEKS)).child(nama))
            .child(div().text_xs().text_color(rgb(WARNA_AKSEN)).child(shortcut))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(move |terminal, _, jendela, konteks| {
                    terminal.menu_split = None;

                    terminal.pecah_panel(arah, jendela, konteks);

                    konteks.stop_propagation();
                    konteks.notify();
                }),
            )
    }
}
