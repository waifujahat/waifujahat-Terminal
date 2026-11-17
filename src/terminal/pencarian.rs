/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Search
 * Purpose: Searches terminal output and manages match navigation
 */

use regex::RegexBuilder;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CakupanPencarian {
    #[default]
    Semua,
    LayarAktif,
    Riwayat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpsiPencarian {
    pub peka_huruf: bool,
    pub kata_utuh: bool,
    pub regex: bool,
}

impl Default for OpsiPencarian {
    fn default() -> Self {
        Self {
            peka_huruf: true,
            kata_utuh: false,
            regex: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HasilPencarian {
    pub baris: usize,
    pub mulai: usize,
    pub selesai: usize,
}

#[derive(Default)]
pub struct PencarianTerminal {
    hasil: Vec<HasilPencarian>,
    indeks_aktif: Option<usize>,
    kesalahan: Option<String>,
}

impl PencarianTerminal {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn perbarui(&mut self, baris: &[String], kata: &str) {
        self.perbarui_bercakupan(baris, kata, 0, CakupanPencarian::Semua);
    }

    pub fn perbarui_bercakupan(
        &mut self,
        baris: &[String],
        kata: &str,
        jumlah_riwayat: usize,
        cakupan: CakupanPencarian,
    ) {
        self.perbarui_dengan_opsi(
            baris,
            kata,
            jumlah_riwayat,
            cakupan,
            OpsiPencarian::default(),
        );
    }

    pub fn perbarui_dengan_opsi(
        &mut self,
        baris: &[String],
        kata: &str,
        jumlah_riwayat: usize,
        cakupan: CakupanPencarian,
        opsi: OpsiPencarian,
    ) {
        let sebelumnya = self.aktif().copied();

        self.cari_dengan_opsi(baris, kata, jumlah_riwayat, cakupan, opsi);

        if let Some(sebelumnya) = sebelumnya
            && !self.hasil.is_empty()
        {
            let indeks = self.hasil.partition_point(|hasil| {
                (hasil.baris, hasil.mulai) < (sebelumnya.baris, sebelumnya.mulai)
            });

            self.indeks_aktif = Some(indeks.min(self.hasil.len() - 1));
        }
    }

    pub fn kesalahan(&self) -> Option<&str> {
        self.kesalahan.as_deref()
    }

    pub fn cari(&mut self, baris: &[String], kata: &str) {
        self.cari_bercakupan(baris, kata, 0, CakupanPencarian::Semua);
    }

    pub fn cari_bercakupan(
        &mut self,
        baris: &[String],
        kata: &str,
        jumlah_riwayat: usize,
        cakupan: CakupanPencarian,
    ) {
        self.cari_dengan_opsi(
            baris,
            kata,
            jumlah_riwayat,
            cakupan,
            OpsiPencarian::default(),
        );
    }

    pub fn cari_dengan_opsi(
        &mut self,
        baris: &[String],
        kata: &str,
        jumlah_riwayat: usize,
        cakupan: CakupanPencarian,
        opsi: OpsiPencarian,
    ) {
        self.bersihkan();

        if kata.is_empty() {
            return;
        }

        let batas_riwayat = jumlah_riwayat.min(baris.len());

        let rentang = match cakupan {
            CakupanPencarian::Semua => 0..baris.len(),

            CakupanPencarian::LayarAktif => batas_riwayat..baris.len(),

            CakupanPencarian::Riwayat => 0..batas_riwayat,
        };

        let pola = if opsi == OpsiPencarian::default() {
            None
        } else {
            let pola = if opsi.regex {
                kata.to_string()
            } else {
                regex::escape(kata)
            };

            let pola = if opsi.kata_utuh {
                format!(r"\b(?:{pola})\b")
            } else {
                pola
            };

            match RegexBuilder::new(&pola)
                .case_insensitive(!opsi.peka_huruf)
                .build()
            {
                Ok(pola) => Some(pola),

                Err(kesalahan) => {
                    self.kesalahan = Some(kesalahan.to_string());

                    return;
                }
            }
        };

        let panjang = kata.chars().count();

        for (indeks, teks) in baris
            .iter()
            .enumerate()
            .take(rentang.end)
            .skip(rentang.start)
        {
            if let Some(pola) = &pola {
                for temuan in pola.find_iter(teks) {
                    if temuan.is_empty() {
                        continue;
                    }

                    self.hasil.push(HasilPencarian {
                        baris: indeks,
                        mulai: teks[..temuan.start()].chars().count(),
                        selesai: teks[..temuan.end()].chars().count(),
                    });
                }
            } else {
                for (posisi, _) in teks.match_indices(kata) {
                    let mulai = teks[..posisi].chars().count();

                    self.hasil.push(HasilPencarian {
                        baris: indeks,
                        mulai,
                        selesai: mulai + panjang,
                    });
                }
            }
        }

        if !self.hasil.is_empty() {
            self.indeks_aktif = Some(0);
        }
    }

    pub fn hasil(&self) -> &[HasilPencarian] {
        &self.hasil
    }

    pub fn hasil_baris(&self, baris: usize) -> &[HasilPencarian] {
        let mulai = self.hasil.partition_point(|hasil| hasil.baris < baris);

        let selesai = self.hasil.partition_point(|hasil| hasil.baris <= baris);

        &self.hasil[mulai..selesai]
    }

    pub fn jumlah(&self) -> usize {
        self.hasil.len()
    }

    pub fn aktif(&self) -> Option<&HasilPencarian> {
        self.indeks_aktif.and_then(|indeks| self.hasil.get(indeks))
    }

    pub fn nomor_aktif(&self) -> Option<usize> {
        self.indeks_aktif.map(|indeks| indeks + 1)
    }

    pub fn berikutnya(&mut self) -> Option<&HasilPencarian> {
        let indeks = self.indeks_aktif?;

        self.indeks_aktif = Some((indeks + 1) % self.hasil.len());

        self.aktif()
    }

    pub fn sebelumnya(&mut self) -> Option<&HasilPencarian> {
        let indeks = self.indeks_aktif?;

        self.indeks_aktif = Some(if indeks == 0 {
            self.hasil.len() - 1
        } else {
            indeks - 1
        });

        self.aktif()
    }

    pub fn bersihkan(&mut self) {
        self.hasil.clear();
        self.indeks_aktif = None;
        self.kesalahan = None;
    }
}
