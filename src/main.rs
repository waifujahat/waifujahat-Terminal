/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Bootstrap
 * Purpose: Initializes the native GPUI window.
 */

mod aset;
mod tampilan;
mod tema;

use gpui::{App, AppContext, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use crate::tampilan::JendelaTerminal;

fn main() {
    gpui_platform::application()
        .with_assets(aset::AsetAplikasi)
        .run(|aplikasi: &mut App| {
            aset::LatarAplikasi::muat(aplikasi).expect("Gagal memuat gambar latar");

            let pengaturan_jendela = WindowOptions {
                window_bounds: Some(WindowBounds::centered(
                    size(px(1120.0), px(630.0)),
                    aplikasi,
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Waifujahat Terminal".into()),
                    ..Default::default()
                }),
                is_resizable: true,
                ..Default::default()
            };

            aplikasi
                .open_window(pengaturan_jendela, |jendela, konteks| {
                    konteks.new(|konteks| JendelaTerminal::baru(jendela, konteks))
                })
                .expect("Gagal membuka Waifujahat Terminal");
        });
}
