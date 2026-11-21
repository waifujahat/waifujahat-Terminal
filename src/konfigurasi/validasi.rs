/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Settings validation
 * Purpose : Validates persistent terminal preferences
 */

use anyhow::{Result, ensure};

use super::PengaturanTerminal;

const BATAS_NAMA_FONT: usize = 128;
const UKURAN_FONT_MINIMUM: f32 = 8.0;
const UKURAN_FONT_MAKSIMUM: f32 = 72.0;

const LEBAR_JENDELA_MINIMUM: u32 = 320;
const LEBAR_JENDELA_MAKSIMUM: u32 = 7680;

const TINGGI_JENDELA_MINIMUM: u32 = 240;
const TINGGI_JENDELA_MAKSIMUM: u32 = 4320;

const BATAS_SCROLLBACK: usize = 100_000;

impl PengaturanTerminal {
    pub fn validasi(&self) -> Result<()> {
        ensure!(
            !self.keluarga_font.trim().is_empty(),
            "font_family tidak boleh kosong"
        );

        ensure!(
            self.keluarga_font.chars().count() <= BATAS_NAMA_FONT,
            "font_family melebihi batas karakter"
        );

        ensure!(
            !self.keluarga_font.chars().any(char::is_control),
            "font_family mengandung karakter kontrol"
        );

        if let Some(ukuran) = self.ukuran_font {
            ensure!(
                (UKURAN_FONT_MINIMUM..=UKURAN_FONT_MAKSIMUM).contains(&ukuran),
                "font_size harus berada antara {} dan {}",
                UKURAN_FONT_MINIMUM,
                UKURAN_FONT_MAKSIMUM
            );
        }

        ensure!(
            (LEBAR_JENDELA_MINIMUM..=LEBAR_JENDELA_MAKSIMUM).contains(&self.lebar_jendela),
            "window_width harus berada antara {} dan {}",
            LEBAR_JENDELA_MINIMUM,
            LEBAR_JENDELA_MAKSIMUM
        );

        ensure!(
            (TINGGI_JENDELA_MINIMUM..=TINGGI_JENDELA_MAKSIMUM).contains(&self.tinggi_jendela),
            "window_height harus berada antara {} dan {}",
            TINGGI_JENDELA_MINIMUM,
            TINGGI_JENDELA_MAKSIMUM
        );

        if let Some(opasitas) = self.opasitas {
            ensure!(
                (0.0..=1.0).contains(&opasitas),
                "opacity harus berada antara 0.0 dan 1.0"
            );
        }

        if let Some(batas) = self.batas_riwayat {
            ensure!(
                batas <= BATAS_SCROLLBACK,
                "scrollback melebihi batas {}",
                BATAS_SCROLLBACK
            );
        }

        if let Some(gambar) = &self.gambar_latar {
            ensure!(
                gambar.is_absolute(),
                "wallpaper harus menggunakan lokasi absolut"
            );

            let format_valid = gambar
                .extension()
                .and_then(|ekstensi| ekstensi.to_str())
                .is_some_and(|ekstensi| {
                    ["jpg", "jpeg", "png"]
                        .iter()
                        .any(|format| ekstensi.eq_ignore_ascii_case(format))
                });

            ensure!(
                format_valid,
                "wallpaper harus menggunakan format JPG, JPEG atau PNG"
            );
        }

        Ok(())
    }
}
