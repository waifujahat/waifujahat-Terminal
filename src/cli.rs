/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Command line
 * Purpose : Handles shell discovery and configuration management
 */

use std::env;

use anyhow::{Result, bail, ensure};

use waifujahat_terminal::konfigurasi::{PengaturanTerminal, PenyimpananPengaturan};

use waifujahat_terminal::terminal::{ProfilShell, deteksi_distribusi_wsl};

pub(crate) fn jalankan_argumen() -> Result<bool> {
    let mut argumen = env::args_os().skip(1);

    let Some(perintah) = argumen.next() else {
        return Ok(false);
    };

    ensure!(argumen.next().is_none(), "Terlalu banyak argumen");

    match perintah.to_str() {
        Some("--help" | "-h") => tampilkan_bantuan(),

        Some("--list-shells") => tampilkan_profil_shell(),

        Some("--settings-path") => {
            let penyimpanan = PenyimpananPengaturan::bawaan()?;

            println!("{}", penyimpanan.jalur().display());
        }

        Some("--init-settings") => {
            let penyimpanan = PenyimpananPengaturan::bawaan()?;

            penyimpanan.muat_atau_buat()?;

            println!("Konfigurasi tersedia : {}", penyimpanan.jalur().display());
        }

        Some("--show-settings") => {
            let penyimpanan = PenyimpananPengaturan::bawaan()?;

            let pengaturan = penyimpanan.muat()?;

            print!("{}", toml::to_string_pretty(&pengaturan)?);
        }

        Some("--validate-settings") => {
            let penyimpanan = PenyimpananPengaturan::bawaan()?;

            penyimpanan.muat()?;

            println!("Konfigurasi valid : {}", penyimpanan.jalur().display());
        }

        _ => {
            bail!("Argumen tidak dikenal : {}", perintah.to_string_lossy());
        }
    }

    Ok(true)
}

fn tampilkan_bantuan() {
    println!("Waifujahat Terminal");
    println!();

    println!("Usage : waifujahat-terminal [option]");
    println!();

    println!("Options");
    println!("  --help                Show available commands");
    println!("  --list-shells         List available shell profiles");
    println!("  --settings-path       Show configuration file path");
    println!("  --init-settings       Initialize default settings");
    println!("  --show-settings       Display terminal settings");
    println!("  --validate-settings   Validate terminal settings");
}

fn muat_pengaturan_shell() -> PengaturanTerminal {
    let hasil = PenyimpananPengaturan::bawaan().and_then(|penyimpanan| {
        if penyimpanan.jalur().try_exists()? {
            penyimpanan.muat()
        } else {
            Ok(PengaturanTerminal::default())
        }
    });

    match hasil {
        Ok(pengaturan) => pengaturan,

        Err(kesalahan) => {
            eprintln!("Gagal memuat konfigurasi : {Error:#}", Error = kesalahan);

            eprintln!("Menggunakan konfigurasi default");

            PengaturanTerminal::default()
        }
    }
}

fn tampilkan_profil_shell() {
    let pengaturan = muat_pengaturan_shell();

    let daftar = ProfilShell::deteksi_tersedia();

    let profil_default = pengaturan
        .shell_default
        .pilih_dengan(|profil| daftar.iter().any(|(tersedia, _)| *tersedia == profil));

    println!("Waifujahat Terminal");
    println!();
    println!("Available shell profiles");
    println!();

    for (profil, lokasi) in &daftar {
        let penanda = if Some(*profil) == profil_default {
            " [Default]"
        } else {
            ""
        };

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
