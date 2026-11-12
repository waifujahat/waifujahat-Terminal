/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Command history
 * Purpose: Stores and navigates submitted commands.
 */

const BATAS_RIWAYAT: usize = 100;

#[derive(Default)]
pub struct RiwayatPerintah {
    daftar: Vec<String>,
    indeks: Option<usize>,
    draf: String,
}

impl RiwayatPerintah {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn catat(&mut self, perintah: String) {
        if !perintah.trim().is_empty() {
            if self.daftar.len() >= BATAS_RIWAYAT {
                self.daftar.remove(0);
            }
            self.daftar.push(perintah);
        }
        self.selesai();
    }

    pub fn sebelumnya(&mut self, masukan: &str) -> Option<String> {
        if self.daftar.is_empty() {
            return None;
        }
        let indeks = match self.indeks {
            Some(indeks) => indeks.saturating_sub(1),
            None => {
                self.draf = masukan.to_string();
                self.daftar.len() - 1
            }
        };
        self.indeks = Some(indeks);
        Some(self.daftar[indeks].clone())
    }

    pub fn berikutnya(&mut self) -> Option<String> {
        let indeks = self.indeks?;
        if indeks + 1 < self.daftar.len() {
            let berikutnya = indeks + 1;
            self.indeks = Some(berikutnya);
            Some(self.daftar[berikutnya].clone())
        } else {
            self.indeks = None;
            Some(std::mem::take(&mut self.draf))
        }
    }

    pub fn selesai(&mut self) {
        self.indeks = None;
        self.draf.clear();
    }
}
