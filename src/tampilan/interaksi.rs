/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Interaction
 * Purpose: Handles keyboard, mouse, clipboard, and command history.
 */

use gpui::{
    ClipboardItem, Context, Font, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    Pixels, Point, TextRun, Window, px, white,
};
use waifujahat_terminal::terminal::{PosisiSeleksi, TombolTerminal};

use super::JendelaTerminal;

use std::time::{Duration, Instant};

impl JendelaTerminal {
    fn posisi_dari_mouse(
        &self,
        posisi: Point<Pixels>,
        jendela: &mut Window,
    ) -> Option<PosisiSeleksi> {
        let baris = self.baris_seleksi();

        let mode_grid = self.mode_grid_aktif();

        let indeks_awal = if mode_grid {
            self.motor.indeks_awal_tampilan()
        } else {
            0
        };

        let jumlah_terlihat = if mode_grid {
            self.motor.ukuran().1
        } else {
            baris.len()
        };

        if baris.is_empty() {
            return None;
        }

        let offset = self.gulir.offset();
        let mut indeks_baris = 0;

        for indeks in 0..jumlah_terlihat {
            let Some(batas) = self.gulir.bounds_for_item(indeks) else {
                continue;
            };

            let atas = batas.top() + offset.y;
            let bawah = batas.bottom() + offset.y;

            if posisi.y < atas {
                break;
            }

            indeks_baris = indeks;

            if posisi.y <= bawah {
                break;
            }
        }

        let batas = self.gulir.bounds_for_item(indeks_baris)?;
        let indeks_seleksi = indeks_awal + indeks_baris;
        let teks = baris.get(indeks_seleksi)?;

        if teks.is_empty() {
            return Some(PosisiSeleksi::baru(indeks_seleksi, 0));
        }

        let font = Font {
            family: "Consolas".into(),
            ..Default::default()
        };

        let run = TextRun {
            len: teks.len(),
            font,
            color: white(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let bentuk = jendela
            .text_system()
            .shape_line(teks.clone().into(), px(14.0), &[run], None);

        let posisi_x = posisi.x - (batas.left() + offset.x);
        let indeks_byte = bentuk.closest_index_for_x(posisi_x).min(teks.len());

        let kolom = teks
            .char_indices()
            .take_while(|(indeks, _)| *indeks < indeks_byte)
            .count();

        Some(PosisiSeleksi::baru(indeks_seleksi, kolom))
    }

    pub(super) fn mouse_ditekan(
        &mut self,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        jendela.focus(&self.fokus, konteks);
        self.arah_gulir_seleksi = 0;

        let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) else {
            self.seleksi.bersihkan();
            konteks.notify();
            return;
        };

        let baris = self.baris_seleksi();

        match kejadian.click_count {
            3.. => {
                self.seleksi.pilih_baris(&baris, posisi.baris);
            }

            2 => {
                self.seleksi.pilih_kata(&baris, posisi);
            }

            _ => {
                self.seleksi.mulai(posisi);
            }
        }

        konteks.notify();
    }
    pub(super) fn mouse_digerakkan(
        &mut self,
        kejadian: &MouseMoveEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if !self.seleksi.sedang_memilih() {
            self.arah_gulir_seleksi = 0;
            return;
        }

        if kejadian.pressed_button != Some(MouseButton::Left) {
            self.arah_gulir_seleksi = 0;
            self.seleksi.selesai();
            return;
        }

        if self.mode_grid_aktif() {
            let offset = self.gulir.offset().y;
            let tinggi = self.motor.ukuran().1;

            let batas_atas = self
                .gulir
                .bounds_for_item(0)
                .map(|batas| batas.top() + offset);

            let batas_bawah = self
                .gulir
                .bounds_for_item(tinggi.saturating_sub(1))
                .map(|batas| batas.bottom() + offset);

            self.arah_gulir_seleksi = match (batas_atas, batas_bawah) {
                (Some(atas), _) if kejadian.position.y < atas => 1,
                (_, Some(bawah)) if kejadian.position.y > bawah => -1,
                _ => 0,
            };
        } else {
            self.arah_gulir_seleksi = 0;
        }

        if let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) {
            self.kolom_gulir_seleksi = posisi.kolom;
            self.seleksi.perbarui(posisi);
            konteks.notify();
        }
    }

    fn salin_seleksi(&mut self, konteks: &mut Context<Self>) {
        let baris = self.baris_seleksi();

        let Some(teks) = self.seleksi.teks_terpilih(&baris) else {
            return;
        };

        konteks.write_to_clipboard(ClipboardItem::new_string(teks));

        self.status = "Copy".into();
        self.batas_status_salin = Some(Instant::now() + Duration::from_secs(3));

        konteks.notify();
    }

    fn tempel_clipboard(&mut self, konteks: &mut Context<Self>) {
        let Some(teks) = konteks.read_from_clipboard().and_then(|isi| isi.text()) else {
            return;
        };

        let teks = teks.replace("\r\n", " ").replace(['\n', '\r'], " ");

        if self.mode_grid_aktif() {
            self.motor.kembali_ke_bawah();
        }

        if teks.is_empty() {
            return;
        }

        self.status = "Paste".into();
        self.batas_status_salin = Some(Instant::now() + Duration::from_secs(3));

        if self.perintah_berjalan || self.motor.layar_alternatif_aktif() {
            self.kirim_input_langsung(TombolTerminal::Teks(&teks));
        } else {
            self.masukan.sisipkan(&teks);
            self.selesai_navigasi_riwayat();
            self.seleksi.bersihkan();
        }

        self.gulir.scroll_to_bottom();
        konteks.notify();
    }

    fn kirim_perintah(&mut self) {
        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            return;
        };

        let perintah = self.masukan.ambil();

        if let Err(kesalahan) = sesi.kirim(&format!("{perintah}\r")) {
            self.masukan.ganti(perintah);
            self.status = format!("Gagal mengirim perintah: {kesalahan}");
            return;
        }

        self.indeks_baris_sebelum_perintah = self
            .layar
            .baris()
            .iter()
            .rposition(|baris| !baris.trim().is_empty())
            .unwrap_or(0);

        self.perintah_berjalan = !perintah.trim().is_empty();
        self.riwayat.catat(perintah);
        self.seleksi.bersihkan();
        self.status.clear();
    }

    fn perintah_sebelumnya(&mut self) {
        if let Some(perintah) = self.riwayat.sebelumnya(self.masukan.teks()) {
            self.masukan.ganti(perintah);
        }
    }

    fn perintah_berikutnya(&mut self) {
        if let Some(perintah) = self.riwayat.berikutnya() {
            self.masukan.ganti(perintah);
        }
    }

    fn selesai_navigasi_riwayat(&mut self) {
        self.riwayat.selesai();
    }

    fn hentikan_perintah(&mut self) {
        self.masukan.bersihkan();
        self.selesai_navigasi_riwayat();
        self.seleksi.bersihkan();

        self.status = match &self.sesi {
            Some(sesi) => match sesi.kirim_tombol(TombolTerminal::Kontrol('c')) {
                Ok(()) => "Esc berhasil".into(),

                Err(kesalahan) => {
                    format!("Gagal mengirim interupsi: {kesalahan}")
                }
            },

            None => "Sesi PowerShell tidak tersedia".into(),
        };
    }

    fn proses_escape(&mut self) {
        self.hentikan_perintah();
    }

    fn kirim_input_langsung(&mut self, tombol: TombolTerminal<'_>) {
        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            return;
        };

        if let Err(kesalahan) = sesi.kirim_tombol(tombol) {
            self.status = format!("Gagal mengirim input: {kesalahan}");
        }
    }

    pub(super) fn proses_tombol(
        &mut self,
        kejadian: &KeyDownEvent,
        _jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        let tombol = &kejadian.keystroke;

        if self.mode_grid_aktif()
            && tombol.modifiers.shift
            && !tombol.modifiers.control
            && !tombol.modifiers.platform
            && !tombol.modifiers.alt
        {
            let tinggi = self.motor.ukuran().1;
            let jumlah_baris = tinggi.saturating_sub(1).max(1);
            let langkah = isize::try_from(jumlah_baris).unwrap_or(isize::MAX);

            let perpindahan = match tombol.key.to_ascii_lowercase().as_str() {
                "pageup" | "page-up" | "page_up" => langkah,
                "pagedown" | "page-down" | "page_down" => -langkah,
                _ => 0,
            };

            if perpindahan != 0 {
                self.motor.geser_riwayat(perpindahan);
                self.seleksi.bersihkan();

                konteks.notify();
                return;
            }
        }

        if tombol.modifiers.control || tombol.modifiers.platform {
            let aplikasi_interaktif = self.perintah_berjalan || self.motor.layar_alternatif_aktif();

            match tombol.key.to_ascii_lowercase().as_str() {
                "c" => {
                    let ada_seleksi = self
                        .seleksi
                        .teks_terpilih(&self.baris_seleksi())
                        .is_some_and(|teks| !teks.is_empty());

                    if ada_seleksi {
                        self.salin_seleksi(konteks);
                    } else if aplikasi_interaktif {
                        self.kirim_input_langsung(TombolTerminal::Kontrol('c'));
                    }
                }

                "v" => {
                    self.tempel_clipboard(konteks);
                }

                "a" => {
                    if self.motor.layar_alternatif_aktif() {
                        self.kirim_input_langsung(TombolTerminal::Kontrol('a'));
                    } else {
                        let baris = self.baris_seleksi();
                        self.seleksi.pilih_semua(&baris);
                    }
                }

                _ => {}
            }

            konteks.notify();
            return;
        }

        if tombol.modifiers.alt {
            return;
        }

        if self.mode_grid_aktif() && self.motor.offset_riwayat() > 0 {
            self.motor.kembali_ke_bawah();
            self.seleksi.bersihkan();
        }

        if self.perintah_berjalan || self.motor.layar_alternatif_aktif() {
            match tombol.key.to_ascii_lowercase().as_str() {
                "escape" | "esc" => {
                    if self.motor.layar_alternatif_aktif() {
                        self.kirim_input_langsung(TombolTerminal::Escape);
                    } else {
                        self.proses_escape();
                    }
                }

                "enter" => {
                    self.kirim_input_langsung(TombolTerminal::Enter);
                }

                "tab" => {
                    self.kirim_input_langsung(TombolTerminal::Tab);
                }

                "backspace" => {
                    self.kirim_input_langsung(TombolTerminal::HapusSebelum);
                }

                "delete" => {
                    self.kirim_input_langsung(TombolTerminal::HapusSesudah);
                }

                "up" | "arrowup" => {
                    self.kirim_input_langsung(TombolTerminal::Atas);
                }

                "down" | "arrowdown" => {
                    self.kirim_input_langsung(TombolTerminal::Bawah);
                }

                "left" | "arrowleft" => {
                    self.kirim_input_langsung(TombolTerminal::Kiri);
                }

                "right" | "arrowright" => {
                    self.kirim_input_langsung(TombolTerminal::Kanan);
                }

                "home" => {
                    self.kirim_input_langsung(TombolTerminal::Awal);
                }

                "end" => {
                    self.kirim_input_langsung(TombolTerminal::Akhir);
                }

                _ => {
                    if let Some(teks) = tombol.key_char.as_deref()
                        && !teks.chars().any(char::is_control)
                    {
                        self.kirim_input_langsung(TombolTerminal::Teks(teks));
                    }
                }
            }

            self.gulir.scroll_to_bottom();
            konteks.notify();
            return;
        }

        match tombol.key.to_ascii_lowercase().as_str() {
            "left" | "arrowleft" => {
                self.masukan.pindah_kiri();
            }

            "right" | "arrowright" => {
                self.masukan.pindah_kanan();
            }

            "home" => {
                self.masukan.pindah_awal();
            }

            "end" => {
                self.masukan.pindah_akhir();
            }

            "backspace" => {
                self.masukan.hapus_sebelum();
                self.selesai_navigasi_riwayat();
            }

            "delete" => {
                self.masukan.hapus_sesudah();
                self.selesai_navigasi_riwayat();
            }

            "up" | "arrowup" => {
                self.perintah_sebelumnya();
            }

            "down" | "arrowdown" => {
                self.perintah_berikutnya();
            }

            "enter" => {
                self.kirim_perintah();
            }

            "escape" | "esc" => {
                self.proses_escape();
            }

            _ => {
                if let Some(karakter) = &tombol.key_char
                    && !karakter.chars().any(char::is_control)
                {
                    self.masukan.sisipkan(karakter);
                    self.selesai_navigasi_riwayat();
                    self.seleksi.bersihkan();
                }
            }
        }

        self.gulir.scroll_to_bottom();
        konteks.notify();
    }
}
