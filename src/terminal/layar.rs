use std::collections::VecDeque;

use vte::{Params, Parser, Perform};

const MAKSIMAL_BARIS: usize = 200;
const MAKSIMAL_KARAKTER: usize = 500;
const TINGGI_LAYAR: usize = 24;

struct BufferTerminal {
    baris: VecDeque<Vec<char>>,
    baris_kursor: usize,
    kolom_kursor: usize,
    awal_layar: usize,
    tinggi_layar: usize,
}

impl Default for BufferTerminal {
    fn default() -> Self {
        Self {
            baris: VecDeque::new(),
            baris_kursor: 0,
            kolom_kursor: 0,
            awal_layar: 0,
            tinggi_layar: TINGGI_LAYAR,
        }
    }
}

impl BufferTerminal {
    fn pastikan_baris(&mut self) {
        while self.baris.len() <= self.baris_kursor {
            self.baris.push_back(Vec::new());

            if self.baris.len() > MAKSIMAL_BARIS {
                self.baris.pop_front();
                self.baris_kursor = self.baris_kursor.saturating_sub(1);
                self.awal_layar = self.awal_layar.saturating_sub(1);
            }
        }
    }

    fn baris_aktif(&mut self) -> &mut Vec<char> {
        self.pastikan_baris();

        self.baris
            .get_mut(self.baris_kursor)
            .expect("Baris terminal tidak tersedia")
    }

    fn tulis(&mut self, karakter: char) {
        let kolom = self.kolom_kursor;

        if kolom >= MAKSIMAL_KARAKTER {
            return;
        }

        let baris = self.baris_aktif();

        if kolom > baris.len() {
            baris.resize(kolom, ' ');
        }

        if kolom == baris.len() {
            baris.push(karakter);
        } else {
            baris[kolom] = karakter;
        }

        self.kolom_kursor += 1;
    }

    fn maju_baris(&mut self) {
        self.baris_kursor += 1;

        if self.baris_kursor >= self.awal_layar + self.tinggi_layar {
            self.awal_layar += 1;
        }

        self.pastikan_baris();
    }

    fn pindahkan_kursor(&mut self, baris: usize, kolom: usize) {
        let baris = baris.max(1).min(self.tinggi_layar);
        let kolom = kolom.max(1).saturating_sub(1);

        self.baris_kursor = self.awal_layar + baris - 1;
        self.kolom_kursor = kolom.min(MAKSIMAL_KARAKTER);

        self.pastikan_baris();
    }

    fn ubah_tinggi(&mut self, tinggi: usize) {
        self.tinggi_layar = tinggi.max(1);

        if self.baris_kursor >= self.awal_layar + self.tinggi_layar {
            self.awal_layar = self.baris_kursor + 1 - self.tinggi_layar;
        }
    }

    fn hapus_karakter(&mut self, jumlah: usize) {
        let kolom = self.kolom_kursor;
        let baris = self.baris_aktif();

        if kolom >= baris.len() {
            return;
        }

        let akhir = kolom.saturating_add(jumlah).min(baris.len());

        for karakter in &mut baris[kolom..akhir] {
            *karakter = ' ';
        }
    }

    fn hapus_baris(&mut self, mode: usize) {
        let kolom = self.kolom_kursor;
        let baris = self.baris_aktif();

        match mode {
            0 => {
                baris.truncate(kolom);
            }

            1 => {
                for karakter in baris.iter_mut().take(kolom.saturating_add(1)) {
                    *karakter = ' ';
                }
            }

            2 => {
                baris.clear();
            }

            _ => {}
        }
    }

    fn hapus_layar(&mut self) {
        self.baris.clear();
        self.baris.push_back(Vec::new());
        self.baris_kursor = 0;
        self.kolom_kursor = 0;
        self.awal_layar = 0;
    }
}

impl Perform for BufferTerminal {
    fn print(&mut self, karakter: char) {
        self.tulis(karakter);
    }

    fn execute(&mut self, kode: u8) {
        match kode {
            b'\r' => {
                self.kolom_kursor = 0;
            }

            b'\n' => {
                self.maju_baris();
            }

            0x08 => {
                self.kolom_kursor = self.kolom_kursor.saturating_sub(1);
            }

            b'\t' => {
                let jumlah = 4 - self.kolom_kursor % 4;

                for _ in 0..jumlah {
                    self.tulis(' ');
                }
            }

            _ => {}
        }
    }

    fn csi_dispatch(&mut self, parameter: &Params, _perantara: &[u8], abaikan: bool, aksi: char) {
        if abaikan {
            return;
        }

        let mut parameter = parameter.iter();

        let pertama = parameter
            .next()
            .and_then(|bagian| bagian.first())
            .copied()
            .unwrap_or(0) as usize;

        let kedua = parameter
            .next()
            .and_then(|bagian| bagian.first())
            .copied()
            .unwrap_or(0) as usize;

        match aksi {
            'H' | 'f' => {
                self.pindahkan_kursor(pertama, kedua);
            }

            'G' => {
                self.kolom_kursor = pertama.max(1).saturating_sub(1).min(MAKSIMAL_KARAKTER);
            }

            'd' => {
                self.pindahkan_kursor(pertama, self.kolom_kursor.saturating_add(1));
            }

            'A' => {
                self.baris_kursor = self
                    .baris_kursor
                    .saturating_sub(pertama.max(1))
                    .max(self.awal_layar);
            }

            'B' => {
                self.baris_kursor = self
                    .baris_kursor
                    .saturating_add(pertama.max(1))
                    .min(self.awal_layar + self.tinggi_layar - 1);

                self.pastikan_baris();
            }

            'C' => {
                self.kolom_kursor = self
                    .kolom_kursor
                    .saturating_add(pertama.max(1))
                    .min(MAKSIMAL_KARAKTER);
            }

            'D' => {
                self.kolom_kursor = self.kolom_kursor.saturating_sub(pertama.max(1));
            }

            'E' => {
                self.baris_kursor = self
                    .baris_kursor
                    .saturating_add(pertama.max(1))
                    .min(self.awal_layar + self.tinggi_layar - 1);

                self.kolom_kursor = 0;
                self.pastikan_baris();
            }

            'F' => {
                self.baris_kursor = self
                    .baris_kursor
                    .saturating_sub(pertama.max(1))
                    .max(self.awal_layar);

                self.kolom_kursor = 0;
            }

            'K' => {
                self.hapus_baris(pertama);
            }

            'X' => {
                self.hapus_karakter(pertama.max(1));
            }

            'J' if pertama == 2 || pertama == 3 => {
                self.hapus_layar();
            }

            _ => {}
        }
    }
}

pub struct LayarTerminal {
    parser: Parser,
    buffer: BufferTerminal,
}

impl Default for LayarTerminal {
    fn default() -> Self {
        Self::baru()
    }
}

impl LayarTerminal {
    pub fn baru() -> Self {
        Self {
            parser: Parser::new(),
            buffer: BufferTerminal::default(),
        }
    }

    pub fn terima(&mut self, keluaran: &[u8]) {
        self.parser.advance(&mut self.buffer, keluaran);
    }

    pub fn ubah_tinggi(&mut self, tinggi: usize) {
        self.buffer.ubah_tinggi(tinggi);
    }

    pub fn baris(&self) -> Vec<String> {
        self.buffer
            .baris
            .iter()
            .map(|baris| baris.iter().collect())
            .collect()
    }
}
