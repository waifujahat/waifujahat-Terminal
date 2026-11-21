/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session monitoring
 * Purpose: Polls isolated PTY sessions and processes terminal output
 */

use std::time::Instant;

use gpui::Context;

use waifujahat_terminal::terminal::{
    PenandaShell, PesanTerminal, ProsesTerminal, SesiTerminal, cari_shell_turunan,
};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

impl RuangSesi {
    fn selesaikan_perintah(&mut self, kode_keluar: Option<i32>) {
        if !self
            .keadaan
            .siklus
            .selesai_dengan_kode(Instant::now(), kode_keluar)
        {
            return;
        }

        let Some(durasi) = self.keadaan.siklus.durasi_terakhir() else {
            return;
        };

        self.status = match self.keadaan.siklus.kode_keluar_terakhir() {
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

    fn perbarui_shell_turunan(&mut self, daftar: &[ProsesTerminal]) {
        let id_proses = self.sesi.as_ref().and_then(SesiTerminal::id_proses);

        let mut hasil = id_proses
            .map(|id| cari_shell_turunan(id, daftar))
            .unwrap_or_default();

        hasil.sort_unstable_by_key(|proses| proses.id);

        if self.shell_turunan == hasil {
            return;
        }

        self.shell_turunan = hasil;

        if std::env::var_os("WAIFUJAHAT_DEBUG_SHELL").is_some() {
            eprintln!("[Shell Monitor] Parent PID : {id_proses:?}");

            if self.shell_turunan.is_empty() {
                eprintln!("[Shell Monitor] Tidak ada nested shell");
            } else {
                for proses in &self.shell_turunan {
                    eprintln!(
                        "[Shell Monitor] {} | PID : {} | Parent : {}",
                        proses.nama_program, proses.id, proses.id_induk
                    );
                }
            }
        }
    }

    fn kirim_respons_terminal(&mut self) {
        while let Some(respons) = self.keadaan.motor.ambil_respons() {
            if let Some(sesi) = &self.sesi
                && let Err(kesalahan) = sesi.kirim_bytes(&respons)
            {
                self.status = format!("Gagal membalas permintaan terminal: {kesalahan}");

                break;
            }
        }
    }

    pub(super) fn periksa_keluaran(&mut self, mode_grid: bool) -> bool {
        let mut berubah = false;

        let layar_alternatif_sebelum = self.keadaan.motor.layar_alternatif_aktif();

        if self
            .keadaan
            .motor
            .periksa_timeout_sinkronisasi(Instant::now())
        {
            if layar_alternatif_sebelum != self.keadaan.motor.layar_alternatif_aktif() {
                self.keadaan.seleksi.bersihkan();
                self.arah_gulir_seleksi = 0;
            }

            self.kirim_respons_terminal();

            berubah = true;
        }

        for _ in 0..64 {
            let pesan = self.sesi.as_ref().and_then(SesiTerminal::try_ambil);

            match pesan {
                Some(PesanTerminal::Keluaran(buffer)) => {
                    let layar_alternatif_sebelum = self.keadaan.motor.layar_alternatif_aktif();

                    let penanda_shell = self.keadaan.terima(&buffer);

                    let layar_alternatif_aktif = self.keadaan.motor.layar_alternatif_aktif();

                    if layar_alternatif_sebelum != layar_alternatif_aktif {
                        self.keadaan.seleksi.bersihkan();
                        self.arah_gulir_seleksi = 0;
                    }

                    for penanda in penanda_shell {
                        if let PenandaShell::PerintahSelesai(kode_keluar) = penanda {
                            self.selesaikan_perintah(kode_keluar);
                        }
                    }

                    if self.keadaan.perintah_selesai_dari_prompt() {
                        self.selesaikan_perintah(None);
                    }

                    self.kirim_respons_terminal();

                    berubah = true;
                }

                Some(PesanTerminal::Kesalahan(pesan)) => {
                    self.status = pesan;
                    self.sesi = None;
                    self.keadaan.siklus.akhiri_sesi();

                    berubah = true;
                    break;
                }

                Some(PesanTerminal::Berakhir) => {
                    self.status = format!("Sesi {} telah berakhir", self.profil.nama());
                    self.sesi = None;
                    self.keadaan.siklus.akhiri_sesi();

                    berubah = true;
                    break;
                }

                None => break,
            }
        }

        if mode_grid && self.keadaan.seleksi.sedang_memilih() && self.arah_gulir_seleksi != 0 {
            let offset_sebelum = self.keadaan.motor.offset_riwayat();

            self.keadaan
                .motor
                .geser_riwayat(isize::from(self.arah_gulir_seleksi));

            if self.keadaan.motor.offset_riwayat() != offset_sebelum {
                let tinggi = self.keadaan.motor.ukuran().1;

                let indeks_baris = if self.arah_gulir_seleksi > 0 {
                    0
                } else {
                    tinggi.saturating_sub(1)
                };

                if let Some(posisi) = self
                    .keadaan
                    .motor
                    .posisi_seleksi_grid(indeks_baris, self.kolom_gulir_seleksi)
                {
                    self.keadaan.seleksi.perbarui(posisi);
                }

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

        if berubah && !self.keadaan.seleksi.sedang_memilih() {
            self.gulir.scroll_to_bottom();
        }

        berubah
    }
}

impl JendelaTerminal {
    pub(super) fn periksa_keluaran(&mut self, konteks: &mut Context<Self>) -> bool {
        let panel_terlihat = self
            .manajer
            .tab_aktif()
            .map(|susunan| susunan.daftar_id())
            .unwrap_or_default();

        let lebar_karakter = self.lebar_karakter;

        let mut berubah = false;

        let potret_shell = match self
            .pemantau_shell
            .as_ref()
            .and_then(|pemantau| pemantau.ambil())
        {
            Some(Ok(daftar)) => Some(daftar),

            Some(Err(kesalahan)) => {
                if std::env::var_os("WAIFUJAHAT_DEBUG_SHELL").is_some() {
                    eprintln!("Gagal memantau proses shell : {kesalahan}");
                }

                None
            }

            None => None,
        };

        for (id, ruang) in self.manajer.iter_sesi_mut() {
            let ukuran_berubah =
                panel_terlihat.contains(&id) && ruang.sesuaikan_ukuran_panel(lebar_karakter);

            let mode_grid = ruang.mode_grid_aktif();

            let generasi_kitty_sebelum = ruang.keadaan.motor.generasi_kitty();

            let generasi_win32_sebelum = ruang.keadaan.motor.generasi_win32();

            let protokol_sebelum = ruang.keadaan.motor.protokol_keyboard();

            let keluaran_berubah = ruang.periksa_keluaran(mode_grid);

            if let Some(daftar) = &potret_shell {
                ruang.perbarui_shell_turunan(daftar);
            }

            let protokol_berubah = protokol_sebelum != ruang.keadaan.motor.protokol_keyboard();

            let kitty_dibatalkan = ruang.sesi.is_none()
                || protokol_berubah
                || generasi_kitty_sebelum != ruang.keadaan.motor.generasi_kitty();

            let win32_dibatalkan = ruang.sesi.is_none()
                || protokol_berubah
                || generasi_win32_sebelum != ruang.keadaan.motor.generasi_win32();

            let mut batalkan_pemilik = false;

            if kitty_dibatalkan && !ruang.tombol_kitty_ditekan.is_empty() {
                ruang.tombol_kitty_ditekan.clear();

                batalkan_pemilik = true;
            }

            if win32_dibatalkan && !ruang.tombol_win32_ditekan.is_empty() {
                ruang.tombol_win32_ditekan.clear();

                batalkan_pemilik = true;
            }

            if batalkan_pemilik {
                self.kepemilikan_tombol.batalkan_pemilik(id);
            }

            if ukuran_berubah || keluaran_berubah {
                ruang.perbarui_pencarian(false);

                berubah = true;
            }
        }

        if berubah {
            konteks.notify();
        }

        true
    }
}
