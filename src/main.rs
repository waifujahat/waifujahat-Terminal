/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : crate
 *
 * Initializes the native application window
 */

mod tampilan;
mod tema;

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use crate::tampilan::JendelaTerminal;

fn main() {
    gpui_platform::application().run(|aplikasi: &mut App| {
        let pengaturan_jendela = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(720.0), px(260.0)), aplikasi)),
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
