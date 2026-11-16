/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session monitoring
 * Purpose: Polls PTY events and updates visible output.
 */

use gpui::Context;
use waifujahat_terminal::terminal::{PenandaShell, PesanTerminal, PosisiSeleksi, SesiTerminal};

use super::JendelaTerminal;

use std::time::Instant;

impl JendelaTerminal {
    fn prompt_powershell_terlihat(&self) -> bool {
        let Some(indeks_baris_awal) = self.siklus.indeks_baris_awal() else {
            return false;
        };

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

        indeks > indeks_baris_awal && teks.starts_with("PS ") && teks.ends_with('>')
    }

    fn selesaikan_perintah(&mut self, kode_keluar: Option<i32>) {
        if !self.siklus.selesai_dengan_kode(Instant::now(), kode_keluar) {
            return;
        }

        let Some(durasi) = self.siklus.durasi_terakhir() else {
            return;
        };

        self.status = match self.siklus.kode_keluar_terakhir() {
            Some(0) => {
                format!(
                    "Berhasil dalam {:.2} detik | Exit code: 0",
                    durasi.as_secs_f64()
                )
            }

            Some(kode) => {
                format!(
                    "Gagal dalam {:.2} detik | Exit code: {}",
                    durasi.as_secs_f64(),
                    kode
                )
            }

            None => {
                format!("Selesai dalam {:.2} detik", durasi.as_secs_f64())
            }
        };
    }

    pub(super) fn periksa_keluaran(&mut self, konteks: &mut Context<Self>) -> bool {
        let mut berubah = false;

        for _ in 0..64 {
            let pesan = self.sesi.as_ref().and_then(SesiTerminal::try_ambil);

            match pesan {
                Some(PesanTerminal::Keluaran(buffer)) => {
                    self.layar.terima(&buffer);

                    let layar_alternatif_sebelum = self.motor.layar_alternatif_aktif();

                    self.motor.terima(&buffer);

                    let layar_alternatif_aktif = self.motor.layar_alternatif_aktif();

                    if layar_alternatif_sebelum != layar_alternatif_aktif {
                        self.seleksi.bersihkan();
                        self.arah_gulir_seleksi = 0;
                    }

                    let penanda_shell = self.integrasi_shell.terima(&buffer);

                    for penanda in penanda_shell {
                        if let PenandaShell::PerintahSelesai(kode_keluar) = penanda {
                            self.selesaikan_perintah(kode_keluar);
                        }
                    }

                    if self.siklus.berjalan()
                        && !layar_alternatif_aktif
                        && self.prompt_powershell_terlihat()
                    {
                        self.selesaikan_perintah(None);
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
                    self.siklus.akhiri_sesi();
                    berubah = true;
                    break;
                }

                Some(PesanTerminal::Berakhir) => {
                    self.status = "Sesi PowerShell telah berakhir".into();
                    self.sesi = None;
                    self.siklus.akhiri_sesi();
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
