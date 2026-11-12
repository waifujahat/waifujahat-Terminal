#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PosisiSeleksi {
    pub baris: usize,
    pub kolom: usize,
}

impl PosisiSeleksi {
    pub fn baru(baris: usize, kolom: usize) -> Self {
        Self { baris, kolom }
    }
}

#[derive(Default)]
pub struct SeleksiTerminal {
    awal: Option<PosisiSeleksi>,
    akhir: Option<PosisiSeleksi>,
    sedang_memilih: bool,
}

impl SeleksiTerminal {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn mulai(&mut self, posisi: PosisiSeleksi) {
        self.awal = Some(posisi);
        self.akhir = Some(posisi);
        self.sedang_memilih = true;
    }

    pub fn perbarui(&mut self, posisi: PosisiSeleksi) {
        if self.sedang_memilih {
            self.akhir = Some(posisi);
        }
    }

    pub fn selesai(&mut self) {
        self.sedang_memilih = false;
    }

    pub fn sedang_memilih(&self) -> bool {
        self.sedang_memilih
    }

    pub fn bersihkan(&mut self) {
        self.awal = None;
        self.akhir = None;
        self.sedang_memilih = false;
    }

    pub fn pilih_semua(&mut self, baris: &[String]) {
        if baris.is_empty() {
            self.bersihkan();
            return;
        }

        let terakhir = baris.len() - 1;
        let kolom_terakhir = baris[terakhir].chars().count();

        self.awal = Some(PosisiSeleksi::baru(0, 0));
        self.akhir = Some(PosisiSeleksi::baru(terakhir, kolom_terakhir));
        self.sedang_memilih = false;
    }

    pub fn pilih_baris(&mut self, baris: &[String], indeks: usize) {
        let Some(teks) = baris.get(indeks) else {
            return;
        };

        self.awal = Some(PosisiSeleksi::baru(indeks, 0));
        self.akhir = Some(PosisiSeleksi::baru(indeks, teks.chars().count()));
        self.sedang_memilih = false;
    }

    pub fn pilih_kata(&mut self, baris: &[String], posisi: PosisiSeleksi) {
        let Some(teks) = baris.get(posisi.baris) else {
            return;
        };

        let karakter: Vec<char> = teks.chars().collect();

        if karakter.is_empty() {
            self.bersihkan();
            return;
        }

        let indeks = posisi.kolom.min(karakter.len() - 1);
        let adalah_kata = |nilai: char| nilai.is_alphanumeric() || nilai == '_';
        let kategori = adalah_kata(karakter[indeks]);

        let mut mulai = indeks;
        let mut selesai = indeks + 1;

        while mulai > 0 && adalah_kata(karakter[mulai - 1]) == kategori {
            mulai -= 1;
        }

        while selesai < karakter.len() && adalah_kata(karakter[selesai]) == kategori {
            selesai += 1;
        }

        self.awal = Some(PosisiSeleksi::baru(posisi.baris, mulai));
        self.akhir = Some(PosisiSeleksi::baru(posisi.baris, selesai));
        self.sedang_memilih = false;
    }

    fn rentang(&self) -> Option<(PosisiSeleksi, PosisiSeleksi)> {
        let awal = self.awal?;
        let akhir = self.akhir?;

        if awal == akhir {
            return None;
        }

        Some(if awal <= akhir {
            (awal, akhir)
        } else {
            (akhir, awal)
        })
    }

    pub fn potongan_baris(&self, indeks: usize, teks: &str) -> Option<(usize, usize)> {
        let (awal, akhir) = self.rentang()?;

        if indeks < awal.baris || indeks > akhir.baris {
            return None;
        }

        let panjang = teks.chars().count();

        let mulai = if indeks == awal.baris {
            awal.kolom.min(panjang)
        } else {
            0
        };

        let selesai = if indeks == akhir.baris {
            akhir.kolom.min(panjang)
        } else {
            panjang
        };

        (mulai < selesai).then_some((mulai, selesai))
    }

    pub fn teks_terpilih(&self, baris: &[String]) -> Option<String> {
        let (awal, akhir) = self.rentang()?;
        let mut hasil = Vec::new();

        for indeks in awal.baris..=akhir.baris {
            let teks = baris.get(indeks)?;
            let panjang = teks.chars().count();

            let mulai = if indeks == awal.baris {
                awal.kolom.min(panjang)
            } else {
                0
            };

            let selesai = if indeks == akhir.baris {
                akhir.kolom.min(panjang)
            } else {
                panjang
            };

            hasil.push(
                teks.chars()
                    .skip(mulai)
                    .take(selesai.saturating_sub(mulai))
                    .collect::<String>(),
            );
        }

        Some(hasil.join("\r\n"))
    }
}
