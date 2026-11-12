/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / ANSI palette
 * Purpose: Converts ANSI color codes to RGB.
 */

use alacritty_terminal::vte::ansi::{Color, NamedColor};

const PALET_ANSI: [u32; 16] = [
    0x222436, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xc0caf5, 0x414868,
    0xff7a93, 0xb9f27c, 0xffd88b, 0x9abaff, 0xd7b2ff, 0xb3eeff, 0xffffff,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotonganWarna {
    pub teks: String,
    pub warna_teks: u32,
    pub warna_latar: u32,
}

fn warna_indeks(indeks: u8) -> u32 {
    match indeks {
        0..=15 => PALET_ANSI[indeks as usize],

        16..=231 => {
            let nilai = (indeks - 16) as u32;

            let merah = nilai / 36;
            let hijau = (nilai / 6) % 6;
            let biru = nilai % 6;

            let komponen = |tingkat: u32| {
                if tingkat == 0 { 0 } else { 55 + tingkat * 40 }
            };

            (komponen(merah) << 16) | (komponen(hijau) << 8) | komponen(biru)
        }

        _ => {
            let abu = 8 + (indeks as u32 - 232) * 10;
            (abu << 16) | (abu << 8) | abu
        }
    }
}

pub(super) fn warna_rgb(warna: Color, teks: u32, latar: u32) -> u32 {
    match warna {
        Color::Spec(rgb) => {
            (u32::from(rgb.r) << 16) | (u32::from(rgb.g) << 8) | u32::from(rgb.b)
        }

        Color::Indexed(indeks) => warna_indeks(indeks),

        Color::Named(nama) => match nama {
            NamedColor::Foreground | NamedColor::BrightForeground => teks,
            NamedColor::Background => latar,
            NamedColor::Cursor => teks,
            NamedColor::DimForeground => redupkan(teks),

            _ => {
                let indeks = nama as usize;

                if indeks < PALET_ANSI.len() {
                    PALET_ANSI[indeks]
                } else {
                    teks
                }
            }
        },
    }
}

fn redupkan(warna: u32) -> u32 {
    let merah = ((warna >> 16) & 0xff) * 7 / 10;
    let hijau = ((warna >> 8) & 0xff) * 7 / 10;
    let biru = (warna & 0xff) * 7 / 10;

    (merah << 16) | (hijau << 8) | biru
}
