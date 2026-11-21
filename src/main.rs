/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Bootstrap
 * Purpose : Initializes the native GPUI window
 */

mod aset;
mod cli;
mod tampilan;
mod tema;

use gpui::{App, AppContext, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use crate::tampilan::{JendelaTerminal, PengaturanAplikasi};

use waifujahat_terminal::konfigurasi::{PengaturanTerminal, PenyimpananPengaturan};

fn main() {
    match cli::jalankan_argumen() {
        Ok(true) => return,

        Ok(false) => {}

        Err(kesalahan) => {
            eprintln!("Gagal menjalankan perintah : {Error:#}", Error = kesalahan);

            std::process::exit(1);
        }
    }

    let pengaturan = muat_pengaturan();

    gpui_platform::application()
        .with_assets(aset::AsetAplikasi)
        .run(move |aplikasi: &mut App| {
            aset::LatarAplikasi::muat_dengan_pengaturan(aplikasi, &pengaturan)
                .expect("Gagal memuat gambar latar");

            let pengaturan_jendela = WindowOptions {
                window_bounds: Some(WindowBounds::centered(
                    size(
                        px(pengaturan.lebar_jendela as f32),
                        px(pengaturan.tinggi_jendela as f32),
                    ),
                    aplikasi,
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Waifujahat Terminal".into()),
                    ..Default::default()
                }),
                is_resizable: true,
                ..Default::default()
            };

            aplikasi.set_global(PengaturanAplikasi(pengaturan));

            aplikasi
                .open_window(pengaturan_jendela, |jendela, konteks| {
                    konteks.new(|konteks| JendelaTerminal::baru(jendela, konteks))
                })
                .expect("Gagal membuka Waifujahat Terminal");
        });
}

fn muat_pengaturan() -> PengaturanTerminal {
    let hasil =
        PenyimpananPengaturan::bawaan().and_then(|penyimpanan| penyimpanan.muat_atau_buat());

    match hasil {
        Ok(pengaturan) => pengaturan,

        Err(kesalahan) => {
            eprintln!("Gagal memuat konfigurasi : {Error:#}", Error = kesalahan);

            eprintln!("Menggunakan konfigurasi default");

            PengaturanTerminal::default()
        }
    }
}
