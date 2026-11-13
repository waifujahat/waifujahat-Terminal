/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Grid snapshot
 * Purpose: Captures visible terminal cells, colors, and cursor position.
 */

use alacritty_terminal::{
    term::cell::Flags,
    vte::ansi::{Color, NamedColor},
};

use super::{MotorTerminal, PotonganWarna, warna::warna_rgb};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelSnapshot {
    pub teks: String,
    pub warna_teks: u32,
    pub warna_latar: u32,
    pub latar_default: bool,
    pub lebar_ganda: bool,
    pub pengisi_lebar_ganda: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarisSnapshot {
    pub teks: String,
    pub potongan: Vec<PotonganWarna>,
    pub sel: Vec<SelSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotTerminal {
    pub kolom: usize,
    pub tinggi: usize,
    pub posisi_kursor: (usize, usize),
    pub baris: Vec<BarisSnapshot>,
}

impl MotorTerminal {
    pub fn ambil_snapshot(&self, warna_teks: u32, warna_latar: u32) -> SnapshotTerminal {
        let (kolom, tinggi) = self.ukuran();

        let baris_teks = self.baris_layar();
        let baris_warna = self.baris_berwarna(warna_teks, warna_latar);

        let baris = baris_teks
            .into_iter()
            .zip(baris_warna)
            .enumerate()
            .map(|(indeks_baris, (teks, potongan))| {
                let sel = (0..kolom)
                    .map(|indeks_kolom| {
                        let sel = self
                            .sel(indeks_baris, indeks_kolom)
                            .expect("Sel grid tidak tersedia");

                        let mut warna_depan = sel.fg;
                        let mut warna_belakang = sel.bg;

                        if sel.flags.contains(Flags::INVERSE) {
                            std::mem::swap(&mut warna_depan, &mut warna_belakang);
                        }

                        let latar_default =
                            matches!(warna_belakang, Color::Named(NamedColor::Background));

                        let warna_latar_sel = warna_rgb(warna_belakang, warna_teks, warna_latar);

                        let warna_teks_sel = if sel.flags.contains(Flags::HIDDEN) {
                            warna_latar_sel
                        } else {
                            warna_rgb(warna_depan, warna_teks, warna_latar)
                        };

                        let pengisi_lebar_ganda = sel.flags.contains(Flags::WIDE_CHAR_SPACER);

                        let teks_sel = if pengisi_lebar_ganda {
                            String::new()
                        } else {
                            let mut teks_sel = sel.c.to_string();

                            if let Some(tambahan) = sel.zerowidth() {
                                teks_sel.extend(tambahan.iter().copied());
                            }

                            teks_sel
                        };

                        SelSnapshot {
                            teks: teks_sel,
                            warna_teks: warna_teks_sel,
                            warna_latar: warna_latar_sel,
                            latar_default,
                            lebar_ganda: sel.flags.contains(Flags::WIDE_CHAR),
                            pengisi_lebar_ganda,
                        }
                    })
                    .collect();

                BarisSnapshot {
                    teks,
                    potongan,
                    sel,
                }
            })
            .collect();

        SnapshotTerminal {
            kolom,
            tinggi,
            posisi_kursor: self.posisi_kursor(),
            baris,
        }
    }
}
