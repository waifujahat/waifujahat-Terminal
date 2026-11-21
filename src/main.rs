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

use waifujahat_terminal::terminal::{ProfilShell, deteksi_distribusi_wsl};

fn main() {
    if std::env::args_os()
        .nth(1)
        .is_some_and(|argumen| argumen == "--list-shells")
    {
        tampilkan_profil_shell();
        return;
    }

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

fn tampilkan_profil_shell() {
    let daftar = ProfilShell::deteksi_tersedia();

    println!("Waifujahat Terminal");
    println!();
    println!("Available shell profiles");
    println!();

    for (indeks, (profil, lokasi)) in daftar.iter().enumerate() {
        let penanda = if indeks == 0 { " [Default]" } else { "" };

        println!("{}{}", profil.nama(), penanda);
        println!("  Path : {}", lokasi.display());
    }

    if daftar.is_empty() {
        println!("Tidak ada shell yang tersedia");
    }

    println!();
    println!("WSL distributions");
    println!();

    match deteksi_distribusi_wsl() {
        Ok(distribusi) if distribusi.is_empty() => {
            println!("Tidak ada distribusi WSL");
        }

        Ok(distribusi) => {
            for nama in distribusi {
                println!("  {nama}");
            }
        }

        Err(kesalahan) => {
            eprintln!("Gagal mendeteksi WSL : {kesalahan}");
        }
    }
}
