/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session monitoring
 * Purpose: Polls isolated PTY sessions and processes terminal output
 */

use std::time::{Duration, Instant};

use gpui::Context;

use waifujahat_terminal::terminal::HasilOutputArsip;
use waifujahat_terminal::terminal::{
    PenandaShell, PesanTerminal, ProsesTerminal, SesiTerminal, StatusProsesTerminal,
    cari_shell_turunan,
};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

const BATAS_PENUTUPAN_SSH: Duration = Duration::from_secs(5);

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
        if self.profil_lokal().is_none() {
            self.shell_turunan.clear();
            return;
        }

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

    fn periksa_penutupan_ssh(&mut self) -> bool {
        if self.profil_lokal().is_some() {
            return false;
        }

        let Some(sesi) = &mut self.sesi else {
            return false;
        };

        let sekarang = Instant::now();

        match sesi.status_proses() {
            Ok(StatusProsesTerminal::Berjalan) => {}

            Ok(StatusProsesTerminal::Berakhir { kode_keluar }) => {
                if self.penutupan_ssh.kode_keluar.is_none() {
                    self.penutupan_ssh.kode_keluar = Some(kode_keluar);

                    self.penutupan_ssh.batas = Some(sekarang + BATAS_PENUTUPAN_SSH);

                    if let Err(kesalahan) = sesi.tutup_pty_setelah_proses_berakhir() {
                        self.penutupan_ssh.kesalahan = Some(format!("{kesalahan:#}"));
                    }
                }
            }

            Err(kesalahan) => {
                self.penutupan_ssh
                    .kesalahan
                    .get_or_insert_with(|| format!("{kesalahan:#}"));

                self.penutupan_ssh
                    .batas
                    .get_or_insert(sekarang + BATAS_PENUTUPAN_SSH);
            }
        }

        let selesai =
            self.penutupan_ssh.kode_keluar.is_some() && self.penutupan_ssh.pembaca_berakhir;

        let kedaluwarsa = self
            .penutupan_ssh
            .batas
            .is_some_and(|batas| sekarang >= batas);

        if !selesai && !kedaluwarsa {
            return false;
        }

        self.status = match self.penutupan_ssh.kode_keluar {
            Some(0) => "Sesi SSH telah berakhir | Exit code: 0".to_owned(),

            Some(kode) => {
                format!("Sesi SSH telah berakhir | Exit code: {kode}")
            }

            None => "Koneksi SSH terputus | Exit code tidak tersedia".to_owned(),
        };

        if !self.penutupan_ssh.pembaca_berakhir {
            self.status.push_str(" | Pembacaan PTY belum selesai");
        }

        if let Some(kesalahan) = &self.penutupan_ssh.kesalahan {
            self.status.push_str(" | ");
            self.status.push_str(kesalahan);
        }

        self.sesi = None;
        self.keadaan.siklus.akhiri_sesi();
        self.mouse_tui_ditekan = None;
        self.arah_gulir_seleksi = 0;
        self.batas_status_salin = None;

        true
    }

    fn proses_efek_keluaran(
        &mut self,
        layar_alternatif_sebelum: bool,
        penanda_shell: Vec<PenandaShell>,
    ) {
        if layar_alternatif_sebelum != self.keadaan.motor.layar_alternatif_aktif() {
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

        if self.profil_lokal().is_some()
            && self.pengendali_arsip.is_some()
            && let Some(sesi) = self.sesi.as_mut()
        {
            match sesi.status_proses() {
                Ok(StatusProsesTerminal::Berakhir { .. }) => {
                    if let Err(kesalahan) = sesi.tutup_pty_setelah_proses_berakhir() {
                        self.status = format!("Gagal menutup PTY lokal: {kesalahan:#}");
                        berubah = true;
                    }
                }
                Ok(StatusProsesTerminal::Berjalan) => {}
                Err(kesalahan) => {
                    self.status = format!("Gagal memeriksa proses lokal: {kesalahan:#}");
                    berubah = true;
                }
            }
        }
        for _ in 0..64 {
            if let Some(pengendali) = self.pengendali_arsip.as_mut() {
                let layar_sebelum = self.keadaan.motor.layar_alternatif_aktif();

                match pengendali.langkah(&mut self.keadaan) {
                    Ok(HasilOutputArsip::TidakAda) => {}

                    Ok(HasilOutputArsip::MenungguArsip) => break,

                    Ok(HasilOutputArsip::ArsipTersimpan(_)) => {
                        berubah = true;
                        continue;
                    }

                    Ok(HasilOutputArsip::OutputDiproses { penanda_shell, .. }) => {
                        self.proses_efek_keluaran(layar_sebelum, penanda_shell);
                        berubah = true;
                        continue;
                    }

                    Ok(HasilOutputArsip::Dihentikan(pesan)) => {
                        self.status = format!("Pengarsipan dihentikan: {pesan}");

                        if let Err(kesalahan) = pengendali.aktifkan_mode_langsung() {
                            self.status = format!("Gagal mengaktifkan fallback: {kesalahan}");
                            berubah = true;
                            break;
                        }

                        berubah = true;
                        continue;
                    }

                    Err(kesalahan) => {
                        self.status = format!("Gagal memproses aliran arsip: {kesalahan:#}");
                        berubah = true;
                        break;
                    }
                }
            }

            let pesan = self
                .antrean_pesan
                .ambil(|| self.sesi.as_ref().and_then(SesiTerminal::try_ambil));

            match pesan {
                Some(PesanTerminal::Keluaran(buffer)) => {
                    if self.penutupan_ssh.batas.is_some() {
                        self.penutupan_ssh.batas = Some(Instant::now() + BATAS_PENUTUPAN_SSH);
                    }
                    if let Some(pengendali) = self.pengendali_arsip.as_mut() {
                        match pengendali.terima_buffer(buffer.clone()) {
                            Ok(true) => continue,

                            Ok(false) => {
                                self.antrean_pesan
                                    .tunda_di_depan(PesanTerminal::Keluaran(buffer));

                                break;
                            }

                            Err(kesalahan) => {
                                self.status = format!(
                                    "Buffer arsip ditolak, output diproses langsung: {kesalahan:#}"
                                );
                            }
                        }
                    }

                    let layar_alternatif_sebelum = self.keadaan.motor.layar_alternatif_aktif();

                    let penanda_shell = self.keadaan.terima(&buffer);

                    self.proses_efek_keluaran(layar_alternatif_sebelum, penanda_shell);

                    berubah = true;
                }

                Some(PesanTerminal::Kesalahan(pesan)) => {
                    if self.profil_lokal().is_none() {
                        self.penutupan_ssh.kesalahan.get_or_insert(pesan);

                        self.penutupan_ssh
                            .batas
                            .get_or_insert(Instant::now() + BATAS_PENUTUPAN_SSH);

                        continue;
                    }

                    self.status = format!("Gagal membaca PTY: {pesan}");

                    berubah = true;
                    continue;
                }

                Some(PesanTerminal::Berakhir) => {
                    if self.profil_lokal().is_none() {
                        self.penutupan_ssh.pembaca_berakhir = true;

                        self.penutupan_ssh
                            .batas
                            .get_or_insert(Instant::now() + BATAS_PENUTUPAN_SSH);

                        break;
                    }

                    if !self.status.starts_with("Gagal membaca PTY:") {
                        self.status = format!("Sesi {} telah berakhir", self.jenis.nama());
                    }

                    self.sesi = None;
                    self.keadaan.siklus.akhiri_sesi();

                    berubah = true;
                    break;
                }

                None => break,
            }
        }

        if self.periksa_penutupan_ssh() {
            berubah = true;
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

        let mut pengendali_selesai = Vec::new();

        for (_, ruang) in self.manajer.iter_sesi_mut() {
            let siap = ruang.sesi.is_none()
                && !ruang.antrean_pesan.ada_tertunda()
                && ruang
                    .pengendali_arsip
                    .as_ref()
                    .is_some_and(|pengendali| !pengendali.ada_pekerjaan_tertunda());

            if siap && let Some(pengendali) = ruang.pengendali_arsip.take() {
                let snapshot =
                    if ruang.profil_lokal().is_some() || ruang.penutupan_ssh.pembaca_berakhir {
                        ruang.keadaan.motor.salin_riwayat_akhir()
                    } else {
                        ruang
                            .status
                            .push_str(" | Arsip akhir dilewati: EOF PTY belum diterima");
                        None
                    };

                pengendali_selesai.push((pengendali, snapshot));
            }
        }

        for (pengendali, snapshot) in pengendali_selesai {
            self.jadwalkan_penyelesaian_arsip(pengendali, snapshot);
            berubah = true;
        }
        let mut kesalahan_penyelesaian_arsip = Vec::new();

        self.tugas_penyelesaian_arsip
            .retain_mut(|tugas| match tugas.periksa() {
                None => true,

                Some(Ok(())) => false,

                Some(Err(kesalahan)) => {
                    kesalahan_penyelesaian_arsip.push(kesalahan);
                    false
                }
            });

        let peringatan_arsip = self
            .tugas_penyelesaian_arsip
            .iter()
            .find_map(|tugas| tugas.peringatan());

        let tugas_arsip_berjalan = !self.tugas_penyelesaian_arsip.is_empty();

        if let Some(ruang) = self.manajer.aktif_mut() {
            match peringatan_arsip {
                Some(pesan) if ruang.status != pesan => {
                    ruang.status = pesan;
                    berubah = true;
                }

                None if ruang.status.starts_with("Pengarsipan menunggu retry:")
                    || ruang.status == "Finalisasi arsip belum selesai setelah 10 detik" =>
                {
                    ruang.status = if tugas_arsip_berjalan {
                        "Finalisasi arsip sedang berlangsung".to_owned()
                    } else {
                        "Finalisasi arsip selesai".to_owned()
                    };

                    berubah = true;
                }

                _ => {}
            }
        }

        for kesalahan in kesalahan_penyelesaian_arsip {
            eprintln!("Gagal menyelesaikan arsip: {kesalahan:#}");

            if let Some(ruang) = self.manajer.aktif_mut() {
                ruang.status = format!("Gagal menyelesaikan arsip: {kesalahan:#}");
                berubah = true;
            }
        }

        if berubah {
            konteks.notify();
        }

        true
    }
}
