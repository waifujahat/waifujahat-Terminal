/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session monitoring
 * Purpose: Polls PTY events and updates visible output.
 */

use gpui::Context;
use waifujahat_terminal::terminal::{PesanTerminal, PosisiSeleksi, SesiTerminal};

use super::JendelaTerminal;

use std::time::Instant;

impl JendelaTerminal {
    fn prompt_powershell_terlihat(&self) -> bool {
        let baris = self.layar.baris();

        let Some((indeks, teks)) = baris
            .iter()
            .enumerate()
            .rev()
            .find(|(_, teks)| !teks.trim().is_empty())
        else {
            return false;
        };

        let teks = teks.trim();

        indeks > self.indeks_baris_sebelum_perintah
            && teks.starts_with("PS ")
            && teks.ends_with('>')
    }

    pub(super) fn periksa_keluaran(&mut self, konteks: &mut Context<Self>) -> bool {
        let mut berubah = false;

        for _ in 0..64 {
            let pesan = self.sesi.as_ref().and_then(SesiTerminal::try_ambil);

            match pesan {
                Some(PesanTerminal::Keluaran(buffer)) => {
                    self.layar.terima(&buffer);
                    self.motor.terima(&buffer);
                    if self.perintah_berjalan && self.prompt_powershell_terlihat() {
                        self.perintah_berjalan = false;
                    }

                    while let Some(respons) = self.motor.ambil_respons() {
                        if let Some(sesi) = &self.sesi
                            && let Err(kesalahan) = sesi.kirim_bytes(&respons)
                        {
                            self.status =
                                format!("Gagal membalas permintaan terminal: {kesalahan}");
                            break;
                        }
                    }

                    berubah = true;
                }

                Some(PesanTerminal::Kesalahan(pesan)) => {
                    self.status = pesan;
                    self.sesi = None;
                    self.perintah_berjalan = false;
                    berubah = true;
                    break;
                }

                Some(PesanTerminal::Berakhir) => {
                    self.status = "Sesi PowerShell telah berakhir".into();
                    self.sesi = None;
                    self.perintah_berjalan = false;
                    berubah = true;
                    break;
                }

                None => break,
            }
        }

        if self.mode_grid_aktif() && self.seleksi.sedang_memilih() && self.arah_gulir_seleksi != 0 {
            let offset_sebelum = self.motor.offset_riwayat();

            self.motor
                .geser_riwayat(isize::from(self.arah_gulir_seleksi));

            if self.motor.offset_riwayat() != offset_sebelum {
                let indeks_awal = self.motor.indeks_awal_tampilan();
                let tinggi = self.motor.ukuran().1;

                let indeks_baris = if self.arah_gulir_seleksi > 0 {
                    indeks_awal
                } else {
                    indeks_awal + tinggi.saturating_sub(1)
                };

                self.seleksi
                    .perbarui(PosisiSeleksi::baru(indeks_baris, self.kolom_gulir_seleksi));

                berubah = true;
            }
        }

        if let Some(batas) = self.batas_status_salin
            && Instant::now() >= batas
        {
            self.batas_status_salin = None;

            if matches!(self.status.as_str(), "Copy" | "Paste") {
                self.status.clear();
                berubah = true;
            }
        }

        if berubah {
            if !self.seleksi.sedang_memilih() {
                self.gulir.scroll_to_bottom();
            }

            konteks.notify();
        }

        self.sesi.is_some()
    }
}
