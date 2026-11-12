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
use waifujahat_terminal::terminal::PosisiSeleksi;

use super::JendelaTerminal;

impl JendelaTerminal {
    fn posisi_dari_mouse(
        &self,
        posisi: Point<Pixels>,
        jendela: &mut Window,
    ) -> Option<PosisiSeleksi> {
        let baris = self.baris_terminal();

        if baris.is_empty() {
            return None;
        }

        let offset = self.gulir.offset();
        let mut indeks_baris = 0;

        for indeks in 0..baris.len() {
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
        let teks = &baris[indeks_baris];

        if teks.is_empty() {
            return Some(PosisiSeleksi::baru(indeks_baris, 0));
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

        Some(PosisiSeleksi::baru(indeks_baris, kolom))
    }

    pub(super) fn mouse_ditekan(
        &mut self,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        jendela.focus(&self.fokus, konteks);

        let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) else {
            self.seleksi.bersihkan();
            konteks.notify();
            return;
        };

        let baris = self.baris_terminal();

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
            return;
        }

        if kejadian.pressed_button != Some(MouseButton::Left) {
            self.seleksi.selesai();
            return;
        }

        if let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) {
            self.seleksi.perbarui(posisi);
            konteks.notify();
        }
    }

    fn salin_seleksi(&mut self, konteks: &mut Context<Self>) {
        let baris = self.baris_terminal();

        let Some(teks) = self.seleksi.teks_terpilih(&baris) else {
            return;
        };

        konteks.write_to_clipboard(ClipboardItem::new_string(teks));

        self.status = "Teks disalin ke clipboard".into();
        konteks.notify();
    }

    fn tempel_clipboard(&mut self, konteks: &mut Context<Self>) {
        let Some(teks) = konteks.read_from_clipboard().and_then(|isi| isi.text()) else {
            return;
        };

        let teks = teks.replace("\r\n", " ").replace(['\n', '\r'], " ");

        self.masukan.sisipkan(&teks);
        self.selesai_navigasi_riwayat();
        self.seleksi.bersihkan();

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

        if let Some(sesi) = &self.sesi {
            if let Err(kesalahan) = sesi.kirim_bytes(b"\x03") {
                self.status = format!("Gagal mengirim interupsi: {kesalahan}");
            } else {
                self.status.clear();
            }
        }
    }

    fn proses_escape(&mut self) {
        let baris = self.baris_terminal();

        if self.seleksi.teks_terpilih(&baris).is_some() {
            self.seleksi.bersihkan();
        } else if !self.masukan.teks().is_empty() {
            self.masukan.bersihkan();
            self.selesai_navigasi_riwayat();
        } else if let Some(sesi) = &self.sesi
            && let Err(kesalahan) = sesi.kirim_bytes(b"\x1b")
        {
            self.status = format!("Gagal mengirim Escape: {kesalahan}");
        }
    }

    pub(super) fn proses_tombol(
        &mut self,
        kejadian: &KeyDownEvent,
        _jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        let tombol = &kejadian.keystroke;

        if tombol.modifiers.control || tombol.modifiers.platform {
            match tombol.key.to_ascii_lowercase().as_str() {
                "c" => {
                    let baris = self.baris_terminal();

                    if self.seleksi.teks_terpilih(&baris).is_some() {
                        self.salin_seleksi(konteks);
                    } else {
                        self.hentikan_perintah();
                    }
                }

                "v" => {
                    self.tempel_clipboard(konteks);
                }

                "a" => {
                    let baris = self.baris_terminal();
                    self.seleksi.pilih_semua(&baris);
                }

                _ => {}
            }

            konteks.notify();
            return;
        }

        if tombol.modifiers.alt {
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
