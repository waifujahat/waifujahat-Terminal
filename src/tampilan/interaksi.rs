/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Interaction
 * Purpose: Handles keyboard, mouse, clipboard and PTY input
 */

use std::time::{Duration, Instant};

use gpui::{
    ClipboardItem, Context, Font, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    Pixels, Point, TextRun, Window, px, white,
};

use waifujahat_terminal::terminal::{AksiPengajuan, PosisiSeleksi, TombolTerminal, sandi_tempel};

use super::JendelaTerminal;

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

        if mode_grid {
            let lebar = Self::lebar_karakter_grid(jendela);

            let posisi_x = f32::from(posisi.x - (batas.left() + offset.x));

            let kolom = self
                .motor
                .indeks_karakter_grid(indeks_baris, posisi_x / lebar)
                .min(teks.chars().count());

            return Some(PosisiSeleksi::baru(indeks_seleksi, kolom));
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

        if self.tangani_tekan_mouse_tui(kejadian, jendela, konteks) {
            return;
        }

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
        if self.mouse_tui_ditekan.is_some() {
            return;
        }

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

        if teks.is_empty() {
            return;
        }

        if self.mode_grid_aktif() {
            self.motor.kembali_ke_bawah();
        }

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            konteks.notify();
            return;
        };

        let berbingkai = self.motor.tempel_berbingkai_aktif();
        let data = sandi_tempel(&teks, berbingkai);

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal menempel teks: {kesalahan}");
            konteks.notify();
            return;
        }

        self.status = "Paste".into();
        self.batas_status_salin = Some(Instant::now() + Duration::from_secs(3));

        self.gulir.scroll_to_bottom();
        konteks.notify();
    }

    fn kirim_perintah_shell(&mut self) {
        if !self.siklus.siap() {
            return;
        }

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            return;
        };

        let indeks_baris = self
            .layar
            .baris()
            .iter()
            .rposition(|baris| !baris.trim().is_empty())
            .unwrap_or(0);

        if let Err(kesalahan) = sesi.kirim_tombol(TombolTerminal::Enter) {
            self.status = format!("Gagal mengirim Enter: {kesalahan}");
            return;
        }

        self.siklus.mulai(indeks_baris, Instant::now());

        self.seleksi.bersihkan();
        self.status.clear();
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

    pub(super) fn pindah_kursor_mouse(&mut self, posisi: PosisiSeleksi) {
        if !self.mode_grid_aktif()
            || self.siklus.berjalan()
            || self.siklus.menunggu_konfirmasi()
            || self.motor.layar_alternatif_aktif()
            || self.motor.offset_riwayat() > 0
        {
            return;
        }

        let (baris_kursor, kolom_kursor) = self.motor.posisi_kursor();

        let indeks_baris = self.motor.jumlah_baris_riwayat() + baris_kursor;

        if posisi.baris != indeks_baris {
            return;
        }

        let baris = self.motor.semua_baris();

        let Some(teks) = baris.get(indeks_baris) else {
            return;
        };

        if !teks.starts_with("PS ")
            || !teks.is_ascii()
            || teks.len() >= self.motor.ukuran().0
            || kolom_kursor > teks.len()
            || baris[indeks_baris + 1..]
                .iter()
                .any(|teks| !teks.trim().is_empty())
        {
            return;
        }

        let Some(awal_input) = teks.find("> ").map(|indeks| indeks + 2) else {
            return;
        };

        if posisi.kolom < awal_input || posisi.kolom > teks.len() || kolom_kursor < awal_input {
            return;
        }

        let (tombol, jumlah) = if posisi.kolom < kolom_kursor {
            (TombolTerminal::Kiri, kolom_kursor - posisi.kolom)
        } else {
            (TombolTerminal::Kanan, posisi.kolom - kolom_kursor)
        };

        if jumlah == 0 {
            return;
        }

        let data: Vec<u8> = std::iter::repeat_n(tombol, jumlah)
            .filter_map(TombolTerminal::ke_bytes)
            .flatten()
            .collect();

        let Some(sesi) = &self.sesi else {
            return;
        };
        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal memindahkan kursor: {kesalahan}");
        }
    }

    fn rentang_seleksi_input(&self) -> Option<(usize, usize)> {
        if !self.mode_grid_aktif()
            || self.siklus.berjalan()
            || self.motor.layar_alternatif_aktif()
            || self.motor.offset_riwayat() > 0
            || self.sesi.is_none()
        {
            return None;
        }

        let (awal, akhir) = self.seleksi.rentang()?;
        let (baris_kursor, kolom_kursor) = self.motor.posisi_kursor();

        let indeks_baris = self.motor.jumlah_baris_riwayat() + baris_kursor;

        if awal.baris != indeks_baris || akhir.baris != indeks_baris {
            return None;
        }

        let baris = self.motor.semua_baris();
        let teks = baris.get(indeks_baris)?;

        if !teks.starts_with("PS ")
            || !teks.is_ascii()
            || teks.len() >= self.motor.ukuran().0
            || kolom_kursor > teks.len()
            || baris[indeks_baris + 1..]
                .iter()
                .any(|teks| !teks.trim().is_empty())
        {
            return None;
        }

        let awal_input = teks.find("> ")? + 2;

        if kolom_kursor < awal_input
            || awal.kolom < awal_input
            || akhir.kolom > teks.len()
            || awal.kolom >= akhir.kolom
        {
            return None;
        }

        Some((awal.kolom - awal_input, akhir.kolom - awal.kolom))
    }

    fn hapus_seleksi_input(&mut self, tombol: TombolTerminal<'_>) -> bool {
        if self.seleksi.rentang().is_none() {
            return false;
        }

        let Some((mulai, panjang)) = self.rentang_seleksi_input() else {
            self.status = "Seleksi bukan bagian dari perintah aktif".into();
            return true;
        };

        let data: Vec<u8> = std::iter::once(TombolTerminal::Awal)
            .chain(std::iter::repeat_n(TombolTerminal::Kanan, mulai))
            .chain(std::iter::repeat_n(TombolTerminal::PilihKanan, panjang))
            .chain(std::iter::once(tombol))
            .filter_map(TombolTerminal::ke_bytes)
            .flatten()
            .collect();

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            return true;
        };

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal menghapus seleksi: {kesalahan}");
            return true;
        }

        self.seleksi.bersihkan();
        self.status.clear();

        true
    }

    fn ajukan_perintah_shell(&mut self) {
        match self.siklus.ajukan() {
            AksiPengajuan::Konfirmasi => {
                self.status.clear();
            }

            AksiPengajuan::Jalankan => {
                self.kirim_perintah_shell();
            }

            AksiPengajuan::Abaikan => {}
        }
    }

    pub(super) fn proses_tombol(
        &mut self,
        kejadian: &KeyDownEvent,
        _jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        let tombol = &kejadian.keystroke;
        let nama_tombol = tombol.key.to_ascii_lowercase();

        let shell_siap = (self.siklus.siap() || self.siklus.menunggu_konfirmasi())
            && !self.motor.layar_alternatif_aktif();

        if shell_siap
            && nama_tombol == "f8"
            && !tombol.modifiers.control
            && !tombol.modifiers.platform
            && !tombol.modifiers.alt
            && !tombol.modifiers.shift
        {
            self.siklus.ubah_konfirmasi();

            self.status = if self.siklus.konfirmasi_aktif() {
                "Konfirmasi menjalankan perintah diaktifkan"
            } else {
                "Konfirmasi menjalankan perintah dinonaktifkan"
            }
            .into();

            konteks.notify();
            return;
        }

        if self.siklus.menunggu_konfirmasi() {
            let tanpa_pengubah =
                !tombol.modifiers.control && !tombol.modifiers.platform && !tombol.modifiers.alt;

            if tanpa_pengubah {
                match nama_tombol.as_str() {
                    "y" => {
                        self.siklus.selesaikan_konfirmasi();
                        self.kirim_perintah_shell();

                        self.gulir.scroll_to_bottom();
                        konteks.notify();
                        return;
                    }

                    "n" | "escape" | "esc" => {
                        self.siklus.selesaikan_konfirmasi();
                        self.kirim_input_langsung(TombolTerminal::Kontrol('c'));

                        self.status = "Perintah tidak dijalankan".into();

                        konteks.notify();
                        return;
                    }

                    "enter" => return,

                    _ => {}
                }
            }

            self.siklus.selesaikan_konfirmasi();
            self.status.clear();
        }

        if self.mode_grid_aktif()
            && tombol.modifiers.shift
            && !tombol.modifiers.control
            && !tombol.modifiers.platform
            && !tombol.modifiers.alt
        {
            let tinggi = self.motor.ukuran().1;
            let jumlah_baris = tinggi.saturating_sub(1).max(1);
            let langkah = isize::try_from(jumlah_baris).unwrap_or(isize::MAX);

            let perpindahan = match nama_tombol.as_str() {
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
            match nama_tombol.as_str() {
                "c" => {
                    let ada_seleksi = self
                        .seleksi
                        .teks_terpilih(&self.baris_seleksi())
                        .is_some_and(|teks| !teks.is_empty());

                    if ada_seleksi {
                        self.salin_seleksi(konteks);
                    } else {
                        self.kirim_input_langsung(TombolTerminal::Kontrol('c'));
                    }
                }

                "v" => {
                    self.tempel_clipboard(konteks);
                }

                "a" => {
                    self.kirim_input_langsung(TombolTerminal::Kontrol('a'));
                }

                "m" | "j" if shell_siap => {
                    self.ajukan_perintah_shell();
                }

                _ if tombol.modifiers.control && !tombol.modifiers.alt => {
                    let mut karakter = tombol.key.chars();

                    if let Some(huruf) = karakter.next()
                        && karakter.next().is_none()
                        && huruf.is_ascii_alphabetic()
                    {
                        self.kirim_input_langsung(TombolTerminal::Kontrol(huruf));
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

        let tombol_hapus = match nama_tombol.as_str() {
            "backspace" => Some(TombolTerminal::HapusSebelum),
            "delete" => Some(TombolTerminal::HapusSesudah),
            _ => None,
        };

        if let Some(tombol_hapus) = tombol_hapus
            && self.hapus_seleksi_input(tombol_hapus)
        {
            konteks.notify();
            return;
        }

        if self.mode_grid_aktif() && self.motor.offset_riwayat() > 0 {
            self.motor.kembali_ke_bawah();
            self.seleksi.bersihkan();
        }

        match nama_tombol.as_str() {
            "escape" | "esc" => {
                self.kirim_input_langsung(TombolTerminal::Escape);
            }

            "enter" if shell_siap => {
                self.ajukan_perintah_shell();
            }

            "enter" => {
                self.kirim_input_langsung(TombolTerminal::Enter);
            }

            "tab" | "backtab" => {
                let tombol_tab = if tombol.modifiers.shift || nama_tombol == "backtab" {
                    TombolTerminal::TabMundur
                } else {
                    TombolTerminal::Tab
                };

                self.kirim_input_langsung(tombol_tab);
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
                let tombol_terminal = if tombol.modifiers.shift {
                    TombolTerminal::PilihKiri
                } else {
                    TombolTerminal::Kiri
                };

                self.kirim_input_langsung(tombol_terminal);
            }

            "right" | "arrowright" => {
                let tombol_terminal = if tombol.modifiers.shift {
                    TombolTerminal::PilihKanan
                } else {
                    TombolTerminal::Kanan
                };

                self.kirim_input_langsung(tombol_terminal);
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
    }
}
