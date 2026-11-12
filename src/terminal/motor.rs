/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Alacritty engine
 * Purpose: Provides ANSI parsing, grid access, and render-ready cell segments.
 */

use super::warna::{PotonganWarna, warna_rgb};

use alacritty_terminal::{
    event::VoidListener,
    grid::Dimensions,
    index::{Column, Line},
    term::{
        Config, Term,
        cell::{Cell, Flags},
        test::TermSize,
    },
    vte::ansi::Processor,
};

pub struct MotorTerminal {
    terminal: Term<VoidListener>,
    parser: Processor,
}

impl MotorTerminal {
    pub fn baru(kolom: usize, baris: usize) -> Self {
        let ukuran = TermSize::new(kolom.max(2), baris.max(1));

        Self {
            terminal: Term::new(Config::default(), &ukuran, VoidListener),
            parser: Processor::new(),
        }
    }

    pub fn terima(&mut self, keluaran: &[u8]) {
        self.parser.advance(&mut self.terminal, keluaran);
    }

    pub fn ubah_ukuran(&mut self, kolom: usize, baris: usize) {
        let ukuran = TermSize::new(kolom.max(2), baris.max(1));
        self.terminal.resize(ukuran);
    }

    pub fn ukuran(&self) -> (usize, usize) {
        (self.terminal.columns(), self.terminal.screen_lines())
    }

    pub fn sel(&self, baris: usize, kolom: usize) -> Option<&Cell> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        if baris >= jumlah_baris || kolom >= jumlah_kolom {
            return None;
        }

        Some(&self.terminal.grid()[Line(baris as i32)][Column(kolom)])
    }

    pub fn baris_layar(&self) -> Vec<String> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        (0..jumlah_baris)
            .map(|baris| {
                let mut teks = String::new();

                for kolom in 0..jumlah_kolom {
                    let Some(sel) = self.sel(baris, kolom) else {
                        continue;
                    };

                    if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                        continue;
                    }

                    teks.push(sel.c);

                    if let Some(tambahan) = sel.zerowidth() {
                        teks.extend(tambahan.iter().copied());
                    }
                }

                teks.trim_end_matches(' ').to_string()
            })
            .collect()
    }

    pub fn baris_berwarna(&self, warna_teks: u32, warna_latar: u32) -> Vec<Vec<PotonganWarna>> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();
        let mut hasil = Vec::with_capacity(jumlah_baris);

        for baris in 0..jumlah_baris {
            let mut potongan: Vec<PotonganWarna> = Vec::new();

            for kolom in 0..jumlah_kolom {
                let Some(sel) = self.sel(baris, kolom) else {
                    continue;
                };

                if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                    continue;
                }

                let mut teks = sel.c.to_string();

                if let Some(tambahan) = sel.zerowidth() {
                    teks.extend(tambahan.iter().copied());
                }

                let mut depan = warna_rgb(sel.fg, warna_teks, warna_latar);

                let mut belakang = warna_rgb(sel.bg, warna_teks, warna_latar);

                if sel.flags.contains(Flags::INVERSE) {
                    std::mem::swap(&mut depan, &mut belakang);
                }

                if sel.flags.contains(Flags::HIDDEN) {
                    depan = belakang;
                }

                if let Some(terakhir) = potongan.last_mut()
                    && terakhir.warna_teks == depan
                    && terakhir.warna_latar == belakang
                {
                    terakhir.teks.push_str(&teks);
                    continue;
                }

                potongan.push(PotonganWarna {
                    teks,
                    warna_teks: depan,
                    warna_latar: belakang,
                });
            }

            while let Some(terakhir) = potongan.last_mut() {
                let panjang = terakhir.teks.trim_end_matches(' ').len();
                terakhir.teks.truncate(panjang);

                if terakhir.teks.is_empty() {
                    potongan.pop();
                } else {
                    break;
                }
            }

            hasil.push(potongan);
        }

        hasil
    }
}
