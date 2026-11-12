use alacritty_terminal::{
    event::VoidListener,
    grid::Dimensions,
    index::{Column, Line},
    term::{
        Config, Term,
        cell::{Cell, Flags},
        test::TermSize,
    },
    vte::ansi::{Color, NamedColor, Processor},
};

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

    fn warna_rgb(warna: Color, teks: u32, latar: u32) -> u32 {
        match warna {
            Color::Spec(rgb) => {
                (u32::from(rgb.r) << 16) | (u32::from(rgb.g) << 8) | u32::from(rgb.b)
            }

            Color::Indexed(indeks) => Self::warna_indeks(indeks),

            Color::Named(nama) => match nama {
                NamedColor::Foreground | NamedColor::BrightForeground => teks,
                NamedColor::Background => latar,
                NamedColor::Cursor => teks,
                NamedColor::DimForeground => Self::redupkan(teks),

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

                let mut depan = Self::warna_rgb(sel.fg, warna_teks, warna_latar);

                let mut belakang = Self::warna_rgb(sel.bg, warna_teks, warna_latar);

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
