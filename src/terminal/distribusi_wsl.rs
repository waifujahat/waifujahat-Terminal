/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / WSL discovery
 * Purpose: Discovers user-accessible WSL distributions
 */

use std::process::Command;

use anyhow::{Context, Result, anyhow};

use super::profil_shell::ProfilShell;

pub fn deteksi_distribusi_wsl() -> Result<Vec<String>> {
    let Some(program) = ProfilShell::program_sistem(r"System32\wsl.exe") else {
        return Ok(Vec::new());
    };

    let mut perintah = Command::new(program);

    perintah.args(["--list", "--quiet"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

        perintah.creation_flags(CREATE_NO_WINDOW);
    }

    let keluaran = perintah
        .output()
        .context("Gagal mendeteksi distribusi WSL")?;

    if !keluaran.status.success() {
        return Err(anyhow!("Deteksi distribusi WSL gagal: {}", keluaran.status));
    }

    urai_distribusi_wsl(&keluaran.stdout)
}

pub fn urai_distribusi_wsl(data: &[u8]) -> Result<Vec<String>> {
    let teks = dekode_keluaran(data)?;

    let mut distribusi = Vec::<String>::new();

    for baris in teks.lines() {
        let nama = baris.trim_matches('\0').trim();

        if nama.is_empty() || distribusi_internal(nama) {
            continue;
        }

        if !distribusi
            .iter()
            .any(|item| item.eq_ignore_ascii_case(nama))
        {
            distribusi.push(nama.to_string());
        }
    }

    Ok(distribusi)
}

fn distribusi_internal(nama: &str) -> bool {
    nama.eq_ignore_ascii_case("docker-desktop") || nama.eq_ignore_ascii_case("docker-desktop-data")
}

fn dekode_keluaran(data: &[u8]) -> Result<String> {
    let utf16 = data.starts_with(&[0xff, 0xfe])
        || data
            .as_chunks::<2>()
            .0
            .iter()
            .any(|pasangan| pasangan[1] == 0);

    if utf16 {
        let data = data.strip_prefix(&[0xff, 0xfe]).unwrap_or(data);

        if !data.len().is_multiple_of(2) {
            return Err(anyhow!("Keluaran UTF-16 tidak valid"));
        }

        let unit: Vec<u16> = data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pasangan| u16::from_le_bytes([pasangan[0], pasangan[1]]))
            .collect();

        return String::from_utf16(&unit).context("Gagal mendekode keluaran UTF-16");
    }

    let teks = std::str::from_utf8(data).context("Gagal mendekode keluaran UTF-8")?;

    Ok(teks.trim_start_matches('\u{feff}').to_string())
}
