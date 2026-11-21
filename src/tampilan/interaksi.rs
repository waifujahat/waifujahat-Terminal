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

use waifujahat_terminal::terminal::{
    AksiPengajuan, PengubahTombol, PosisiSeleksi, RekamanTombolWin32, TombolKhusus, TombolTerminal,
    sandi_tempel,
};

use super::{JendelaTerminal, ruang_sesi::RuangSesi};

pub(super) fn petakan_tombol_khusus(nama: &str) -> Option<TombolKhusus> {
    let tombol = match nama {
        "up" | "arrowup" => TombolKhusus::Atas,
        "down" | "arrowdown" => TombolKhusus::Bawah,
        "left" | "arrowleft" => TombolKhusus::Kiri,
        "right" | "arrowright" => TombolKhusus::Kanan,
        "home" => TombolKhusus::Awal,
        "end" => TombolKhusus::Akhir,
        "insert" => TombolKhusus::Sisipkan,
        "delete" => TombolKhusus::HapusSesudah,

        "pageup" | "page-up" | "page_up" => TombolKhusus::HalamanAtas,

        "pagedown" | "page-down" | "page_down" => TombolKhusus::HalamanBawah,

        _ => {
            let nomor = nama.strip_prefix('f')?.parse::<u8>().ok()?;

            if !(1..=12).contains(&nomor) {
                return None;
            }

            TombolKhusus::Fungsi(nomor)
        }
    };

    Some(tombol)
}

impl RuangSesi {
    pub(super) fn posisi_dari_mouse(
        &self,
        posisi: Point<Pixels>,
        jendela: &mut Window,
    ) -> Option<PosisiSeleksi> {
        let mode_grid = self.mode_grid_aktif();

        let baris = if mode_grid {
            Vec::new()
        } else {
            self.baris_seleksi()
        };

        let jumlah_terlihat = if mode_grid {
            self.keadaan.motor.ukuran().1
        } else {
            baris.len()
        };

        if jumlah_terlihat == 0 {
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

        let posisi_x = posisi.x - (batas.left() + offset.x);

        if mode_grid {
            let lebar = Self::lebar_karakter_grid(jendela);

            return self
                .keadaan
                .motor
                .posisi_seleksi_grid(indeks_baris, f32::from(posisi_x) / lebar);
        }

        let teks = baris.get(indeks_baris)?;

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
        konteks: &mut Context<JendelaTerminal>,
    ) {
        self.arah_gulir_seleksi = 0;

        if self.tangani_tekan_mouse_tui(kejadian, jendela, konteks) {
            return;
        }

        let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) else {
            self.keadaan.seleksi.bersihkan();

            konteks.notify();

            return;
        };

        match kejadian.click_count {
            3.. => {
                let baris = self.baris_seleksi();

                self.keadaan.seleksi.pilih_baris(&baris, posisi.baris);
            }

            2 => {
                let baris = self.baris_seleksi();

                self.keadaan.seleksi.pilih_kata(&baris, posisi);
            }

            _ => {
                self.keadaan.seleksi.mulai(posisi);
            }
        }

        konteks.notify();
    }

    pub(super) fn mouse_digerakkan(
        &mut self,
        kejadian: &MouseMoveEvent,
        jendela: &mut Window,
        konteks: &mut Context<JendelaTerminal>,
    ) {
        if self.mouse_tui_ditekan.is_some() {
            return;
        }

        if !self.keadaan.seleksi.sedang_memilih() {
            self.arah_gulir_seleksi = 0;

            return;
        }

        if kejadian.pressed_button != Some(MouseButton::Left) {
            self.arah_gulir_seleksi = 0;

            self.keadaan.seleksi.selesai();

            return;
        }

        if self.mode_grid_aktif() {
            let offset = self.gulir.offset();
            let tinggi = self.keadaan.motor.ukuran().1;

            let batas_pertama = self.gulir.bounds_for_item(0);

            let batas_atas = batas_pertama.as_ref().map(|batas| batas.top() + offset.y);

            let batas_bawah = self
                .gulir
                .bounds_for_item(tinggi.saturating_sub(1))
                .map(|batas| batas.bottom() + offset.y);

            self.arah_gulir_seleksi = match (batas_atas, batas_bawah) {
                (Some(atas), _) if kejadian.position.y < atas => {
                    langkah_gulir_seleksi(f32::from(atas - kejadian.position.y))
                }

                (_, Some(bawah)) if kejadian.position.y > bawah => {
                    -langkah_gulir_seleksi(f32::from(kejadian.position.y - bawah))
                }

                _ => 0,
            };

            if let Some(batas) = batas_pertama {
                let posisi_x = kejadian.position.x - (batas.left() + offset.x);

                self.kolom_gulir_seleksi = f32::from(posisi_x) / Self::lebar_karakter_grid(jendela);
            }
        } else {
            self.arah_gulir_seleksi = 0;
        }

        if let Some(posisi) = self.posisi_dari_mouse(kejadian.position, jendela) {
            self.keadaan.seleksi.perbarui(posisi);

            konteks.notify();
        }
    }

    fn salin_seleksi(&mut self, konteks: &mut Context<JendelaTerminal>) {
        let baris = self.baris_seleksi();

        let Some(teks) = self.keadaan.seleksi.teks_terpilih(&baris) else {
            return;
        };

        konteks.write_to_clipboard(ClipboardItem::new_string(teks));

        self.status = "Copy".into();
        self.batas_status_salin = Some(Instant::now() + Duration::from_secs(3));

        konteks.notify();
    }

    fn tempel_clipboard(&mut self, konteks: &mut Context<JendelaTerminal>) {
        let Some(teks) = konteks.read_from_clipboard().and_then(|isi| isi.text()) else {
            return;
        };

        if teks.is_empty() {
            return;
        }

        if self.mode_grid_aktif() {
            self.keadaan.motor.kembali_ke_bawah();
        }

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi terminal tidak tersedia".into();
            konteks.notify();
            return;
        };

        let berbingkai = self.keadaan.motor.tempel_berbingkai_aktif();
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
        if !self.keadaan.siklus.siap() {
            return;
        }

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi terminal tidak tersedia".into();
            return;
        };

        let indeks_baris = self
            .keadaan
            .layar
            .baris()
            .iter()
            .rposition(|baris| !baris.trim().is_empty())
            .unwrap_or(0);

        let Some(bytes) = self.keadaan.motor.sandi_tombol(TombolTerminal::Enter) else {
            return;
        };

        if let Err(kesalahan) = sesi.kirim_bytes(&bytes) {
            self.status = format!("Gagal mengirim Enter: {kesalahan}");
            return;
        }

        self.keadaan.siklus.mulai(indeks_baris, Instant::now());

        self.keadaan.seleksi.bersihkan();
        self.status.clear();
    }

    fn kirim_input_langsung(&mut self, tombol: TombolTerminal<'_>) {
        let Some(sesi) = &self.sesi else {
            self.status = "Sesi terminal tidak tersedia".into();
            return;
        };

        if let Some(bytes) = self.keadaan.motor.sandi_tombol(tombol)
            && let Err(kesalahan) = sesi.kirim_bytes(&bytes)
        {
            self.status = format!("Gagal mengirim input: {kesalahan}");
        }
    }

    pub(super) fn pindah_kursor_mouse(&mut self, posisi: PosisiSeleksi) {
        if !self.profil.mendukung_integrasi_shell()
            || !self.mode_grid_aktif()
            || self.keadaan.siklus.berjalan()
            || self.keadaan.siklus.menunggu_konfirmasi()
            || self.keadaan.motor.layar_alternatif_aktif()
            || self.keadaan.motor.offset_riwayat() > 0
        {
            return;
        }

        let (baris_kursor, kolom_kursor) = self.keadaan.motor.posisi_kursor();

        let indeks_baris = self.keadaan.motor.jumlah_baris_riwayat() + baris_kursor;

        if posisi.baris != indeks_baris {
            return;
        }

        let baris = self.keadaan.motor.semua_baris();

        let Some(teks) = baris.get(indeks_baris) else {
            return;
        };

        if !teks.starts_with("PS ")
            || !teks.is_ascii()
            || teks.len() >= self.keadaan.motor.ukuran().0
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
            .filter_map(|tombol| self.keadaan.motor.sandi_tombol(tombol))
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
            || self.keadaan.siklus.berjalan()
            || self.keadaan.motor.layar_alternatif_aktif()
            || self.keadaan.motor.offset_riwayat() > 0
            || self.sesi.is_none()
        {
            return None;
        }

        let (awal, akhir) = self.keadaan.seleksi.rentang()?;
        let (baris_kursor, kolom_kursor) = self.keadaan.motor.posisi_kursor();

        let indeks_baris = self.keadaan.motor.jumlah_baris_riwayat() + baris_kursor;

        if awal.baris != indeks_baris || akhir.baris != indeks_baris {
            return None;
        }

        let baris = self.keadaan.motor.semua_baris();
        let teks = baris.get(indeks_baris)?;

        if !teks.starts_with("PS ")
            || !teks.is_ascii()
            || teks.len() >= self.keadaan.motor.ukuran().0
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
        if !self.profil.mendukung_integrasi_shell() || self.keadaan.seleksi.rentang().is_none() {
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
            .filter_map(|tombol| self.keadaan.motor.sandi_tombol(tombol))
            .flatten()
            .collect();

        let Some(sesi) = &self.sesi else {
            self.status = "Sesi terminal tidak tersedia".into();
            return true;
        };

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal menghapus seleksi: {kesalahan}");
            return true;
        }

        self.keadaan.seleksi.bersihkan();
        self.status.clear();

        true
    }

    fn ajukan_perintah_shell(&mut self) {
        match self.keadaan.siklus.ajukan() {
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
        rekaman_win32: Option<RekamanTombolWin32>,
        _jendela: &mut Window,
        konteks: &mut Context<JendelaTerminal>,
    ) {
        let tombol = &kejadian.keystroke;
        let nama_tombol = tombol.key.to_ascii_lowercase();

        let shell_siap = self.profil.mendukung_integrasi_shell()
            && self.sesi.is_some()
            && (self.keadaan.siklus.siap() || self.keadaan.siklus.menunggu_konfirmasi())
            && !self.keadaan.motor.layar_alternatif_aktif();

        if shell_siap
            && nama_tombol == "f8"
            && !tombol.modifiers.control
            && !tombol.modifiers.platform
            && !tombol.modifiers.alt
            && !tombol.modifiers.shift
        {
            self.keadaan.siklus.ubah_konfirmasi();

            self.status = if self.keadaan.siklus.konfirmasi_aktif() {
                "Konfirmasi menjalankan perintah diaktifkan"
            } else {
                "Konfirmasi menjalankan perintah dinonaktifkan"
            }
            .into();

            konteks.notify();
            return;
        }

        if self.keadaan.siklus.menunggu_konfirmasi() {
            let tanpa_pengubah =
                !tombol.modifiers.control && !tombol.modifiers.platform && !tombol.modifiers.alt;

            if tanpa_pengubah {
                match nama_tombol.as_str() {
                    "y" => {
                        self.keadaan.siklus.selesaikan_konfirmasi();
                        self.kirim_perintah_shell();

                        self.gulir.scroll_to_bottom();
                        konteks.notify();
                        return;
                    }

                    "n" | "escape" | "esc" => {
                        self.keadaan.siklus.selesaikan_konfirmasi();
                        self.kirim_input_langsung(TombolTerminal::Kontrol('c'));

                        self.status = "Perintah tidak dijalankan".into();

                        konteks.notify();
                        return;
                    }

                    "enter" => return,

                    _ => {}
                }
            }

            self.keadaan.siklus.selesaikan_konfirmasi();
            self.status.clear();
        }

        if self.mode_grid_aktif()
            && tombol.modifiers.shift
            && !tombol.modifiers.control
            && !tombol.modifiers.platform
            && !tombol.modifiers.alt
        {
            let tinggi = self.keadaan.motor.ukuran().1;
            let jumlah_baris = tinggi.saturating_sub(1).max(1);
            let langkah = isize::try_from(jumlah_baris).unwrap_or(isize::MAX);

            let perpindahan = match nama_tombol.as_str() {
                "pageup" | "page-up" | "page_up" => langkah,
                "pagedown" | "page-down" | "page_down" => -langkah,
                _ => 0,
            };

            if perpindahan != 0 {
                self.keadaan.motor.geser_riwayat(perpindahan);
                self.keadaan.seleksi.bersihkan();

                konteks.notify();
                return;
            }
        }

        if self.keadaan.motor.kitty_aktif() {
            if kejadian.prefer_character_input
                && let Some(teks) = tombol.key_char.as_deref()
                && !teks.chars().any(char::is_control)
            {
                self.kirim_input_langsung(TombolTerminal::Teks(teks));
                konteks.notify();
                return;
            }

            let shortcut_internal = tombol.modifiers.control
                && !tombol.modifiers.alt
                && (nama_tombol == "v"
                    || (nama_tombol == "c" && self.keadaan.seleksi.rentang().is_some())
                    || (shell_siap && matches!(nama_tombol.as_str(), "m" | "j")));

            if !shortcut_internal && self.kirim_tombol_kitty(kejadian) {
                self.gulir.scroll_to_bottom();
                konteks.notify();
                return;
            }
        }

        if !tombol.modifiers.platform
            && let Some(tombol_khusus) = petakan_tombol_khusus(&nama_tombol)
        {
            if tombol_khusus == TombolKhusus::HapusSesudah
                && !tombol.modifiers.control
                && !tombol.modifiers.alt
                && self.hapus_seleksi_input(TombolTerminal::HapusSesudah)
            {
                konteks.notify();
                return;
            }

            if self.mode_grid_aktif() && self.keadaan.motor.offset_riwayat() > 0 {
                self.keadaan.motor.kembali_ke_bawah();
                self.keadaan.seleksi.bersihkan();
            }

            if !kejadian.is_held
                && let Some(rekaman) = rekaman_win32
                && self.tekan_tombol_win32(rekaman)
            {
                self.gulir.scroll_to_bottom();

                konteks.stop_propagation();
                konteks.notify();

                return;
            }

            let pengubah = PengubahTombol {
                shift: tombol.modifiers.shift,
                alt: tombol.modifiers.alt,
                kontrol: tombol.modifiers.control,
            };

            self.kirim_input_langsung(TombolTerminal::Khusus(tombol_khusus, pengubah));

            self.gulir.scroll_to_bottom();
            konteks.notify();

            return;
        }

        let shortcut_kontrol_internal = matches!(nama_tombol.as_str(), "c" | "v")
            || (shell_siap && matches!(nama_tombol.as_str(), "m" | "j"));

        if tombol.modifiers.control
            && !tombol.modifiers.alt
            && !tombol.modifiers.platform
            && !shortcut_kontrol_internal
            && !kejadian.is_held
            && let Some(rekaman) = rekaman_win32
            && matches!(rekaman.kode_virtual, 0x41..=0x5A)
            && rekaman.unicode == rekaman.kode_virtual - 0x40
            && rekaman.status_kontrol & 0x000C != 0
            && self.tekan_tombol_win32(rekaman)
        {
            self.gulir.scroll_to_bottom();

            konteks.stop_propagation();
            konteks.notify();

            return;
        }

        if tombol.modifiers.control || tombol.modifiers.platform {
            match nama_tombol.as_str() {
                "c" => {
                    let ada_seleksi = self
                        .keadaan
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
            if !tombol.modifiers.control
                && !tombol.modifiers.platform
                && !kejadian.is_held
                && let Some(rekaman) = rekaman_win32
                && matches!(rekaman.kode_virtual, 0x41..=0x5A)
                && matches!(rekaman.unicode, 0x41..=0x5A | 0x61..=0x7A)
                && rekaman.status_kontrol & 0x0003 != 0
                && rekaman.status_kontrol & 0x000C == 0
                && self.tekan_tombol_win32(rekaman)
            {
                self.gulir.scroll_to_bottom();

                konteks.stop_propagation();
                konteks.notify();
            }

            return;
        }

        if nama_tombol == "backspace" && self.hapus_seleksi_input(TombolTerminal::HapusSebelum) {
            konteks.notify();
            return;
        }

        if self.mode_grid_aktif() && self.keadaan.motor.offset_riwayat() > 0 {
            self.keadaan.motor.kembali_ke_bawah();
            self.keadaan.seleksi.bersihkan();
        }

        let masukan_win32 = rekaman_win32.filter(|rekaman| match nama_tombol.as_str() {
            "enter" | "return" => !shell_siap && rekaman.kode_virtual == 0x0D,

            "tab" | "backtab" => rekaman.kode_virtual == 0x09,

            "backspace" => rekaman.kode_virtual == 0x08,

            "escape" | "esc" => rekaman.kode_virtual == 0x1B,

            _ => {
                matches!(rekaman.unicode, 0x20..=0x7E)
                    && tombol
                        .key_char
                        .as_deref()
                        .is_none_or(|teks| teks.as_bytes() == [rekaman.unicode as u8])
            }
        });

        if !kejadian.is_held
            && let Some(rekaman) = masukan_win32
            && self.tekan_tombol_win32(rekaman)
        {
            self.gulir.scroll_to_bottom();

            konteks.stop_propagation();
            konteks.notify();

            return;
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

fn langkah_gulir_seleksi(jarak: f32) -> i8 {
    if jarak <= 0.0 {
        0
    } else if jarak <= 8.0 {
        1
    } else if jarak <= 16.0 {
        3
    } else if jarak <= 24.0 {
        6
    } else {
        12
    }
}
