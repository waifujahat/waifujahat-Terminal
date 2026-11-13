/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Alacritty engine
 * Purpose: Provides ANSI parsing, grid access, and render-ready cell segments.
 */

use super::warna::{PotonganWarna, warna_rgb};

use std::sync::mpsc::{self, Receiver, Sender};

use alacritty_terminal::{
    event::{Event, EventListener},
    grid::{Dimensions, Scroll},
    index::{Column, Line},
    term::{
        Config, Term, TermMode,
        cell::{Cell, Flags},
        test::TermSize,
    },
    vte::ansi::Processor,
};

struct PengirimRespons {
    pengirim: Sender<Vec<u8>>,
}

impl EventListener for PengirimRespons {
    fn send_event(&self, kejadian: Event) {
        if let Event::PtyWrite(teks) = kejadian {
            let _ = self.pengirim.send(teks.into_bytes());
        }
    }
}

pub struct MotorTerminal {
    terminal: Term<PengirimRespons>,
    parser: Processor,
    penerima_respons: Receiver<Vec<u8>>,
}

impl MotorTerminal {
    pub fn baru(kolom: usize, baris: usize) -> Self {
        let ukuran = TermSize::new(kolom.max(2), baris.max(1));
        let (pengirim_respons, penerima_respons) = mpsc::channel();

        Self {
            terminal: Term::new(
                Config::default(),
                &ukuran,
                PengirimRespons {
                    pengirim: pengirim_respons,
                },
            ),
            parser: Processor::new(),
            penerima_respons,
        }
    }

    pub fn terima(&mut self, keluaran: &[u8]) {
        self.parser.advance(&mut self.terminal, keluaran);
    }

    pub fn ambil_respons(&self) -> Option<Vec<u8>> {
        self.penerima_respons.try_recv().ok()
    }

    pub fn ubah_ukuran(&mut self, kolom: usize, baris: usize) {
        let ukuran = TermSize::new(kolom.max(2), baris.max(1));
        self.terminal.resize(ukuran);
    }

    pub fn ukuran(&self) -> (usize, usize) {
        (self.terminal.columns(), self.terminal.screen_lines())
    }

    pub fn posisi_kursor(&self) -> (usize, usize) {
        let posisi = self.terminal.grid().cursor.point;

        (usize::try_from(posisi.line.0).unwrap_or(0), posisi.column.0)
    }

    pub fn kursor_terlihat(&self) -> bool {
        self.terminal.mode().contains(TermMode::SHOW_CURSOR)
    }

    pub fn jumlah_baris_riwayat(&self) -> usize {
        self.terminal.grid().history_size()
    }

    pub fn baris_riwayat(&self, indeks: usize) -> Option<String> {
        let grid = self.terminal.grid();
        let jumlah_riwayat = grid.history_size();

        if indeks >= jumlah_riwayat {
            return None;
        }

        let jarak = jumlah_riwayat - indeks;
        let nomor_baris = i32::try_from(jarak).ok()?;
        let baris = &grid[Line(-nomor_baris)];

        let mut teks = String::new();

        for kolom in 0..self.terminal.columns() {
            let sel = &baris[Column(kolom)];

            if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            teks.push(sel.c);

            if let Some(tambahan) = sel.zerowidth() {
                teks.extend(tambahan.iter().copied());
            }
        }

        Some(teks.trim_end_matches(' ').to_string())
    }

    pub fn offset_riwayat(&self) -> usize {
        self.terminal.grid().display_offset()
    }

    pub fn geser_riwayat(&mut self, jumlah: isize) {
        let jumlah = i32::try_from(jumlah).unwrap_or_else(|_| {
            if jumlah.is_negative() {
                i32::MIN
            } else {
                i32::MAX
            }
        });

        if jumlah != 0 {
            self.terminal.scroll_display(Scroll::Delta(jumlah));
        }
    }

    pub fn kembali_ke_bawah(&mut self) {
        self.terminal.scroll_display(Scroll::Bottom);
    }

    pub fn sel(&self, baris: usize, kolom: usize) -> Option<&Cell> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        if baris >= jumlah_baris || kolom >= jumlah_kolom {
            return None;
        }

        let offset = i32::try_from(self.offset_riwayat()).ok()?;
        let indeks = i32::try_from(baris).ok()?.checked_sub(offset)?;

        Some(&self.terminal.grid()[Line(indeks)][Column(kolom)])
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
