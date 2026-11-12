/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Input buffer
 * Purpose: Edits local text input and cursor position.
 */

#[derive(Default)]
pub struct MasukanTerminal {
    teks: String,
    posisi: usize,
}

impl MasukanTerminal {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn teks(&self) -> &str {
        &self.teks
    }

    pub fn posisi(&self) -> usize {
        self.posisi
    }

    fn indeks_byte(&self, posisi: usize) -> usize {
        self.teks
            .char_indices()
            .nth(posisi)
            .map(|(indeks, _)| indeks)
            .unwrap_or(self.teks.len())
    }

    pub fn sisipkan(&mut self, teks: &str) {
        let indeks = self.indeks_byte(self.posisi);

        self.teks.insert_str(indeks, teks);
        self.posisi += teks.chars().count();
    }

    pub fn hapus_sebelum(&mut self) {
        if self.posisi == 0 {
            return;
        }

        let awal = self.indeks_byte(self.posisi - 1);
        let akhir = self.indeks_byte(self.posisi);

        self.teks.replace_range(awal..akhir, "");
        self.posisi -= 1;
    }

    pub fn hapus_sesudah(&mut self) {
        if self.posisi >= self.teks.chars().count() {
            return;
        }

        let awal = self.indeks_byte(self.posisi);
        let akhir = self.indeks_byte(self.posisi + 1);

        self.teks.replace_range(awal..akhir, "");
    }

    pub fn pindah_kiri(&mut self) {
        self.posisi = self.posisi.saturating_sub(1);
    }

    pub fn pindah_kanan(&mut self) {
        self.posisi = (self.posisi + 1).min(self.teks.chars().count());
    }

    pub fn pindah_awal(&mut self) {
        self.posisi = 0;
    }

    pub fn pindah_akhir(&mut self) {
        self.posisi = self.teks.chars().count();
    }

    pub fn ganti(&mut self, teks: String) {
        self.posisi = teks.chars().count();
        self.teks = teks;
    }

    pub fn bersihkan(&mut self) {
        self.teks.clear();
        self.posisi = 0;
    }

    pub fn ambil(&mut self) -> String {
        self.posisi = 0;
        std::mem::take(&mut self.teks)
    }

    pub fn tampilkan(&self, awalan: &str) -> String {
        let indeks = self.indeks_byte(self.posisi);

        format!("{awalan}{}▏{}", &self.teks[..indeks], &self.teks[indeks..])
    }
}
