/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Alacritty engine
 * Purpose: Provides ANSI parsing, grid access and render-ready cell segments
 */

use super::warna::{PotonganWarna, warna_rgb};

use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use alacritty_terminal::{
    event::{Event, EventListener},
    grid::{Dimensions, Scroll},
    index::{Column, Line},
    term::{
        Config, Term, TermMode,
        cell::{Cell, Flags},
        test::TermSize,
    },
    vte::ansi::{Color, NamedColor, Processor},
};

use super::PerintahModeKitty;
use super::keyboard_kitty::{JenisAksiTombol, sandi_kitty_tombol, sandi_kitty_unicode};
use super::keyboard_win32::RekamanTombolWin32;
use super::mode_win32::ModeInputWin32;
use super::negosiasi_kitty::NegosiasiKitty;
use super::protokol_keyboard::ProtokolKeyboard;
use super::seleksi::PosisiSeleksi;
use super::tombol::{PengubahTombol, TombolTerminal};
use std::time::Instant;

struct PengirimRespons {
    pengirim: Sender<Vec<u8>>,
}

impl EventListener for PengirimRespons {
    fn send_event(&self, kejadian: Event) {
        if let Event::PtyWrite(teks) = kejadian {
            let _ = self.pengirim.send(teks.into_bytes());
        }
    }
}

pub struct MotorTerminal {
    terminal: Term<PengirimRespons>,
    batas_riwayat: usize,
    parser: Processor,
    pengirim_respons: Sender<Vec<u8>>,
    penerima_respons: Receiver<Vec<u8>>,
    negosiasi_kitty: Option<NegosiasiKitty>,
    generasi_kitty: u64,
    mode_win32: ModeInputWin32,
    generasi_riwayat: u64,
}

impl MotorTerminal {
    pub fn baru(kolom: usize, baris: usize) -> Self {
        Self::buat(kolom, baris, false, None)
    }

    pub fn generasi_riwayat(&self) -> u64 {
        self.generasi_riwayat
    }

    pub fn batas_riwayat(&self) -> usize {
        self.batas_riwayat
    }

    pub fn cadangkan_riwayat_sebelum_output(&mut self, jumlah_byte: usize) -> bool {
        self.cadangkan_riwayat_dengan_batas(jumlah_byte, 16_384)
    }

    fn cadangkan_riwayat_dengan_batas(
        &mut self,
        jumlah_byte: usize,
        batas_tambahan: usize,
    ) -> bool {
        const BATAS_POTONGAN: usize = 64;

        if jumlah_byte == 0 || jumlah_byte > BATAS_POTONGAN {
            return false;
        }

        if self.layar_alternatif_aktif() {
            return true;
        }

        let (_, tinggi) = self.ukuran();

        let Some(cadangan) = jumlah_byte.checked_mul(tinggi) else {
            return false;
        };

        if cadangan > batas_tambahan {
            return false;
        }

        let Some(kapasitas_maksimum) = self.batas_riwayat.checked_add(batas_tambahan) else {
            return false;
        };

        let Some(kapasitas_diperlukan) = self.jumlah_baris_riwayat().checked_add(cadangan) else {
            return false;
        };

        if kapasitas_diperlukan > kapasitas_maksimum {
            return false;
        }

        let kapasitas = self.batas_riwayat.max(kapasitas_diperlukan);

        self.terminal.grid_mut().update_history(kapasitas);

        true
    }

    fn tandai_perubahan_riwayat(&mut self) {
        self.generasi_riwayat = self.generasi_riwayat.wrapping_add(1);
    }

    pub fn baru_dengan_kitty(kolom: usize, baris: usize) -> Self {
        Self::buat(kolom, baris, true, None)
    }

    pub fn baru_dengan_kitty_dan_riwayat(
        kolom: usize,
        baris: usize,
        batas_riwayat: Option<usize>,
    ) -> Self {
        Self::buat(kolom, baris, true, batas_riwayat)
    }

    fn buat(kolom: usize, baris: usize, kitty: bool, batas_riwayat: Option<usize>) -> Self {
        let ukuran = TermSize::new(kolom.max(2), baris.max(1));

        let (pengirim_respons, penerima_respons) = mpsc::channel();

        let mut konfigurasi = Config {
            kitty_keyboard: false,
            ..Config::default()
        };

        if let Some(batas) = batas_riwayat {
            konfigurasi.scrolling_history = batas;
        }

        let batas_riwayat = konfigurasi.scrolling_history;

        Self {
            terminal: Term::new(
                konfigurasi,
                &ukuran,
                PengirimRespons {
                    pengirim: pengirim_respons.clone(),
                },
            ),
            batas_riwayat,
            parser: Processor::new(),
            pengirim_respons,
            penerima_respons,
            negosiasi_kitty: kitty.then(NegosiasiKitty::baru),
            generasi_kitty: 0,
            mode_win32: ModeInputWin32::baru(),
            generasi_riwayat: 0,
        }
    }

    pub fn generasi_kitty(&self) -> u64 {
        self.generasi_kitty
    }

    pub fn generasi_win32(&self) -> u64 {
        self.mode_win32.generasi()
    }

    pub fn win32_aktif(&self) -> bool {
        self.mode_win32.aktif()
    }

    pub fn protokol_keyboard(&self) -> ProtokolKeyboard {
        if self.kitty_aktif() {
            ProtokolKeyboard::Kitty
        } else if self.win32_aktif() {
            ProtokolKeyboard::Win32
        } else {
            ProtokolKeyboard::Xterm
        }
    }

    pub fn sandi_rekaman_win32(&self, rekaman: RekamanTombolWin32) -> Option<Vec<u8>> {
        if self.protokol_keyboard() != ProtokolKeyboard::Win32 {
            return None;
        }

        Some(rekaman.sandi())
    }

    pub fn kursor_aplikasi_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::APP_CURSOR)
    }

    pub fn sandi_tombol(&self, tombol: TombolTerminal<'_>) -> Option<Vec<u8>> {
        self.sandi_tombol_dengan_aksi(tombol, JenisAksiTombol::Tekan)
    }

    pub fn sandi_tombol_dengan_aksi(
        &self,
        tombol: TombolTerminal<'_>,
        aksi: JenisAksiTombol,
    ) -> Option<Vec<u8>> {
        let disambiguasi = self.kitty_disambiguasi_aktif();
        let laporkan_aksi = self.kitty_pelaporan_aksi_aktif();

        if disambiguasi || (laporkan_aksi && aksi != JenisAksiTombol::Tekan) {
            return sandi_kitty_tombol(tombol, aksi, laporkan_aksi);
        }

        if aksi == JenisAksiTombol::Lepas {
            return None;
        }

        let hasil = tombol.ke_bytes_dengan_kursor_aplikasi(self.kursor_aplikasi_aktif());

        if laporkan_aksi && hasil.is_none() {
            return sandi_kitty_tombol(tombol, aksi, laporkan_aksi);
        }

        hasil
    }

    pub fn sandi_unicode_kitty(
        &self,
        karakter: char,
        pengubah: PengubahTombol,
        aksi: JenisAksiTombol,
    ) -> Option<Vec<u8>> {
        if !self.kitty_aktif() {
            return None;
        }

        let laporkan = self.kitty_pelaporan_aksi_aktif();

        if self.kitty_disambiguasi_aktif() || aksi != JenisAksiTombol::Tekan {
            return sandi_kitty_unicode(karakter, pengubah, aksi, laporkan);
        }

        let kode = match karakter.to_ascii_lowercase() {
            '\x1b' => Some(0x1b),

            huruf @ 'a'..='z' if pengubah.kontrol => Some(huruf.to_ascii_uppercase() as u8 - b'@'),

            ' ' | '@' if pengubah.kontrol => Some(0),

            simbol @ '['..='_' if pengubah.kontrol => Some(simbol as u8 - b'@'),

            '?' if pengubah.kontrol => Some(0x7f),

            _ if pengubah.alt
                && !pengubah.kontrol
                && (karakter.is_ascii_graphic() || karakter == ' ') =>
            {
                let karakter = if pengubah.shift {
                    match karakter {
                        '`' => '~',
                        '1' => '!',
                        '2' => '@',
                        '3' => '#',
                        '4' => '$',
                        '5' => '%',
                        '6' => '^',
                        '7' => '&',
                        '8' => '*',
                        '9' => '(',
                        '0' => ')',
                        '-' => '_',
                        '=' => '+',
                        '[' => '{',
                        ']' => '}',
                        '\\' => '|',
                        ';' => ':',
                        '\'' => '"',
                        ',' => '<',
                        '.' => '>',
                        '/' => '?',
                        lainnya => lainnya.to_ascii_uppercase(),
                    }
                } else {
                    karakter
                };

                Some(karakter as u8)
            }

            _ => None,
        };

        if let Some(kode) = kode {
            let mut hasil = Vec::new();

            if pengubah.alt {
                hasil.push(0x1b);
            }

            hasil.push(kode);

            return Some(hasil);
        }

        sandi_kitty_unicode(karakter, pengubah, aksi, laporkan)
    }

    pub fn kitty_disambiguasi_aktif(&self) -> bool {
        self.negosiasi_kitty.as_ref().is_some_and(|negosiasi| {
            negosiasi
                .mode(self.layar_alternatif_aktif())
                .disambiguasi_aktif()
        })
    }

    pub fn kitty_pelaporan_aksi_aktif(&self) -> bool {
        self.negosiasi_kitty.as_ref().is_some_and(|negosiasi| {
            negosiasi
                .mode(self.layar_alternatif_aktif())
                .pelaporan_aksi_aktif()
        })
    }

    pub fn kitty_aktif(&self) -> bool {
        self.kitty_disambiguasi_aktif() || self.kitty_pelaporan_aksi_aktif()
    }

    pub fn terima(&mut self, keluaran: &[u8]) {
        if !keluaran.is_empty() {
            self.tandai_perubahan_riwayat();
        }

        self.periksa_timeout_sinkronisasi(Instant::now());

        self.mode_win32.terima(keluaran);

        let Some(negosiasi) = self.negosiasi_kitty.as_mut() else {
            self.parser.advance(&mut self.terminal, keluaran);
            return;
        };

        let mut sisa = keluaran;

        while !sisa.is_empty() {
            let (jumlah, perintah) = negosiasi.terima(sisa);

            let alternatif_sebelum = self.terminal.mode().contains(TermMode::ALT_SCREEN);

            self.parser.advance(&mut self.terminal, &sisa[..jumlah]);

            sisa = &sisa[jumlah..];

            if alternatif_sebelum != self.terminal.mode().contains(TermMode::ALT_SCREEN) {
                self.generasi_kitty = self.generasi_kitty.wrapping_add(1);
            }

            if let Some(perintah) = perintah {
                if perintah == PerintahModeKitty::PeralihanLayar {
                    continue;
                }

                if perintah != PerintahModeKitty::Tanya {
                    self.generasi_kitty = self.generasi_kitty.wrapping_add(1);
                }

                if self.parser.sync_bytes_count() > 0 {
                    let alternatif_sebelum = self.terminal.mode().contains(TermMode::ALT_SCREEN);

                    self.parser.stop_sync(&mut self.terminal);

                    if alternatif_sebelum != self.terminal.mode().contains(TermMode::ALT_SCREEN) {
                        self.generasi_kitty = self.generasi_kitty.wrapping_add(1);
                    }
                }

                let alternatif = self.terminal.mode().contains(TermMode::ALT_SCREEN);

                if let Some(respons) = negosiasi.proses(perintah, alternatif) {
                    let _ = self.pengirim_respons.send(respons);
                }
            }
        }
    }

    pub fn periksa_timeout_sinkronisasi(&mut self, sekarang: Instant) -> bool {
        let kedaluwarsa = self
            .parser
            .sync_timeout()
            .sync_timeout()
            .is_some_and(|batas| sekarang >= batas);

        if !kedaluwarsa {
            return false;
        }

        self.tandai_perubahan_riwayat();

        let alternatif_sebelum = self.layar_alternatif_aktif();

        self.parser.stop_sync(&mut self.terminal);

        if self.negosiasi_kitty.is_some() && alternatif_sebelum != self.layar_alternatif_aktif() {
            self.generasi_kitty = self.generasi_kitty.wrapping_add(1);
        }

        true
    }

    pub fn ambil_respons(&self) -> Option<Vec<u8>> {
        self.penerima_respons.try_recv().ok()
    }

    pub fn ubah_ukuran(&mut self, kolom: usize, baris: usize) {
        self.tandai_perubahan_riwayat();

        let kolom = kolom.max(2);
        let baris = baris.max(1);

        let (kolom_sebelum, baris_sebelum) = self.ukuran();

        let pertahankan_viewport = cfg!(windows)
            && !self.layar_alternatif_aktif()
            && self.jumlah_baris_riwayat() > 0
            && (kolom != kolom_sebelum || baris > baris_sebelum);

        let penanda = if pertahankan_viewport {
            let grid = self.terminal.grid_mut();
            let sel = &mut grid[Line(0)][Column(0)];

            let atribut_asli = sel.extra.take();

            let identitas = Arc::new(atribut_asli.as_deref().cloned().unwrap_or_default());

            sel.extra = Some(Arc::clone(&identitas));

            Some((identitas, atribut_asli))
        } else {
            None
        };

        self.terminal.resize(TermSize::new(kolom, baris));

        let Some((identitas, atribut_asli)) = penanda else {
            return;
        };

        let grid = self.terminal.grid_mut();

        let mut posisi = None;

        'cari: for indeks_baris in grid.topmost_line().0..baris as i32 {
            for indeks_kolom in 0..kolom {
                let sel = &grid[Line(indeks_baris)][Column(indeks_kolom)];

                if sel
                    .extra
                    .as_ref()
                    .is_some_and(|atribut| Arc::ptr_eq(atribut, &identitas))
                {
                    posisi = Some((indeks_baris, indeks_kolom));

                    break 'cari;
                }
            }
        }

        let Some((indeks_baris, indeks_kolom)) = posisi else {
            return;
        };

        grid[Line(indeks_baris)][Column(indeks_kolom)].extra = atribut_asli;

        if indeks_baris <= 0 {
            return;
        }

        grid.scroll_up(&(Line(0)..Line(baris as i32)), indeks_baris as usize);

        grid.cursor.point.line = Line((grid.cursor.point.line.0 - indeks_baris).max(0));

        grid.saved_cursor.point.line = Line((grid.saved_cursor.point.line.0 - indeks_baris).max(0));
    }

    pub fn ukuran(&self) -> (usize, usize) {
        (self.terminal.columns(), self.terminal.screen_lines())
    }

    pub fn posisi_kursor(&self) -> (usize, usize) {
        let posisi = self.terminal.grid().cursor.point;

        (usize::try_from(posisi.line.0).unwrap_or(0), posisi.column.0)
    }

    pub fn kursor_terlihat(&self) -> bool {
        self.terminal.mode().contains(TermMode::SHOW_CURSOR)
    }

    pub fn layar_alternatif_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::ALT_SCREEN)
    }

    pub fn tempel_berbingkai_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::BRACKETED_PASTE)
    }

    pub fn pelaporan_mouse_aktif(&self) -> bool {
        self.terminal.mode().intersects(TermMode::MOUSE_MODE)
    }

    pub fn mouse_sgr_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::SGR_MOUSE)
    }

    pub fn pelaporan_drag_mouse_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::MOUSE_DRAG)
    }

    pub fn pelaporan_gerakan_mouse_aktif(&self) -> bool {
        self.terminal.mode().contains(TermMode::MOUSE_MOTION)
    }

    pub fn jumlah_baris_riwayat(&self) -> usize {
        self.terminal.grid().history_size()
    }

    pub fn baris_riwayat(&self, indeks: usize) -> Option<String> {
        let grid = self.terminal.grid();
        let jumlah_riwayat = grid.history_size();

        if indeks >= jumlah_riwayat {
            return None;
        }

        let jarak = jumlah_riwayat - indeks;
        let nomor_baris = i32::try_from(jarak).ok()?;
        let baris = &grid[Line(-nomor_baris)];

        let mut teks = String::new();

        for kolom in 0..self.terminal.columns() {
            let sel = &baris[Column(kolom)];

            if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            teks.push(sel.c);

            if let Some(tambahan) = sel.zerowidth() {
                teks.extend(tambahan.iter().copied());
            }
        }

        Some(teks.trim_end_matches(' ').to_string())
    }

    pub fn salin_riwayat_akhir(&self) -> Option<Vec<String>> {
        if self.layar_alternatif_aktif() {
            return None;
        }

        (0..self.jumlah_baris_riwayat())
            .map(|indeks| self.baris_riwayat(indeks))
            .collect()
    }
    pub fn siapkan_batch_arsip(&self, jumlah: usize) -> Vec<String> {
        const BATAS_BATCH: usize = 64;

        if self.layar_alternatif_aktif() {
            return Vec::new();
        }

        let jumlah = jumlah.min(BATAS_BATCH).min(self.jumlah_baris_riwayat());

        (0..jumlah)
            .map(|indeks| self.baris_riwayat(indeks))
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default()
    }

    fn konfirmasi_batch_arsip(&mut self, jumlah: usize) -> bool {
        const BATAS_BATCH: usize = 64;

        let jumlah_riwayat = self.jumlah_baris_riwayat();

        if jumlah == 0
            || jumlah > BATAS_BATCH
            || jumlah > jumlah_riwayat
            || self.layar_alternatif_aktif()
        {
            return false;
        }

        let jumlah_tersisa = jumlah_riwayat - jumlah;
        let kapasitas_pemulihan = self.batas_riwayat.max(jumlah_tersisa);

        let grid = self.terminal.grid_mut();

        grid.update_history(jumlah_tersisa);
        grid.update_history(kapasitas_pemulihan);

        self.tandai_perubahan_riwayat();

        true
    }

    pub fn konfirmasi_batch_arsip_terverifikasi(
        &mut self,
        generasi: u64,
        baris: &[String],
    ) -> bool {
        if generasi != self.generasi_riwayat || baris.is_empty() || self.layar_alternatif_aktif() {
            return false;
        }

        let baris_sekarang = self.siapkan_batch_arsip(baris.len());

        if baris_sekarang != baris {
            return false;
        }

        self.konfirmasi_batch_arsip(baris.len())
    }

    pub fn offset_riwayat(&self) -> usize {
        self.terminal.grid().display_offset()
    }

    pub fn geser_riwayat(&mut self, jumlah: isize) {
        let jumlah = i32::try_from(jumlah).unwrap_or_else(|_| {
            if jumlah.is_negative() {
                i32::MIN
            } else {
                i32::MAX
            }
        });

        if jumlah != 0 {
            self.terminal.scroll_display(Scroll::Delta(jumlah));
        }
    }

    pub fn kembali_ke_bawah(&mut self) {
        self.terminal.scroll_display(Scroll::Bottom);
    }

    pub fn sel(&self, baris: usize, kolom: usize) -> Option<&Cell> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        if baris >= jumlah_baris || kolom >= jumlah_kolom {
            return None;
        }

        let offset = i32::try_from(self.offset_riwayat()).ok()?;
        let indeks = i32::try_from(baris).ok()?.checked_sub(offset)?;

        Some(&self.terminal.grid()[Line(indeks)][Column(kolom)])
    }

    pub fn baris_layar(&self) -> Vec<String> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        (0..jumlah_baris)
            .map(|baris| {
                let mut teks = String::new();

                for kolom in 0..jumlah_kolom {
                    let Some(sel) = self.sel(baris, kolom) else {
                        continue;
                    };

                    if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                        continue;
                    }

                    teks.push(sel.c);

                    if let Some(tambahan) = sel.zerowidth() {
                        teks.extend(tambahan.iter().copied());
                    }
                }

                teks.trim_end_matches(' ').to_string()
            })
            .collect()
    }

    pub fn indeks_awal_tampilan(&self) -> usize {
        self.jumlah_baris_riwayat()
            .saturating_sub(self.offset_riwayat())
    }

    pub fn tampilkan_baris(&mut self, indeks: usize) -> bool {
        let jumlah_riwayat = self.jumlah_baris_riwayat();
        let (_, tinggi) = self.ukuran();

        if indeks >= jumlah_riwayat.saturating_add(tinggi) {
            return false;
        }

        let awal = self.indeks_awal_tampilan();
        let akhir = awal.saturating_add(tinggi);

        let target_awal = if indeks < awal {
            indeks
        } else if indeks >= akhir {
            indeks.saturating_sub(tinggi.saturating_sub(1))
        } else {
            return true;
        };

        let target_offset = jumlah_riwayat.saturating_sub(target_awal);

        let Ok(target) = isize::try_from(target_offset) else {
            return false;
        };

        let Ok(offset) = isize::try_from(self.offset_riwayat()) else {
            return false;
        };

        self.geser_riwayat(target - offset);

        true
    }

    pub fn semua_baris(&self) -> Vec<String> {
        let grid = self.terminal.grid();

        let Ok(jumlah_riwayat) = i32::try_from(grid.history_size()) else {
            return Vec::new();
        };

        let Ok(tinggi) = i32::try_from(self.terminal.screen_lines()) else {
            return Vec::new();
        };

        let jumlah_kolom = self.terminal.columns();
        let mut hasil = Vec::new();

        for indeks in -jumlah_riwayat..tinggi {
            let baris = &grid[Line(indeks)];
            let mut teks = String::new();

            for kolom in 0..jumlah_kolom {
                let sel = &baris[Column(kolom)];

                if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                    continue;
                }

                teks.push(sel.c);

                if let Some(tambahan) = sel.zerowidth() {
                    teks.extend(tambahan.iter().copied());
                }
            }

            hasil.push(teks.trim_end_matches(' ').to_string());
        }

        hasil
    }

    pub fn indeks_karakter_grid(&self, baris: usize, posisi: f32) -> usize {
        let (jumlah_kolom, _) = self.ukuran();
        let mut indeks_karakter = 0;

        for kolom in 0..jumlah_kolom {
            let Some(sel) = self.sel(baris, kolom) else {
                break;
            };

            if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            let lebar = if sel.flags.contains(Flags::WIDE_CHAR) {
                2.0
            } else {
                1.0
            };

            if posisi < kolom as f32 + lebar / 2.0 {
                return indeks_karakter;
            }

            indeks_karakter += 1 + sel.zerowidth().map_or(0, |tambahan| tambahan.len());
        }

        indeks_karakter
    }

    pub fn posisi_seleksi_grid(&self, baris: usize, posisi: f32) -> Option<PosisiSeleksi> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();

        if baris >= jumlah_baris {
            return None;
        }

        let mut teks = String::new();

        for kolom in 0..jumlah_kolom {
            let sel = self.sel(baris, kolom)?;

            if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            teks.push(sel.c);

            if let Some(tambahan) = sel.zerowidth() {
                teks.extend(tambahan.iter().copied());
            }
        }

        let panjang = teks.trim_end_matches(' ').chars().count();

        let kolom = self.indeks_karakter_grid(baris, posisi).min(panjang);

        Some(PosisiSeleksi::baru(
            self.indeks_awal_tampilan() + baris,
            kolom,
        ))
    }

    pub fn baris_berwarna(&self, warna_teks: u32, warna_latar: u32) -> Vec<Vec<PotonganWarna>> {
        let (jumlah_kolom, jumlah_baris) = self.ukuran();
        let mut hasil = Vec::with_capacity(jumlah_baris);

        for baris in 0..jumlah_baris {
            let mut potongan: Vec<PotonganWarna> = Vec::new();

            for kolom in 0..jumlah_kolom {
                let Some(sel) = self.sel(baris, kolom) else {
                    continue;
                };

                if sel.flags.contains(Flags::WIDE_CHAR_SPACER) {
                    continue;
                }

                let mut teks = sel.c.to_string();

                if let Some(tambahan) = sel.zerowidth() {
                    teks.extend(tambahan.iter().copied());
                }

                let mut warna_depan = sel.fg;
                let mut warna_belakang = sel.bg;

                if sel.flags.contains(Flags::INVERSE) {
                    std::mem::swap(&mut warna_depan, &mut warna_belakang);
                }

                let latar_default = matches!(warna_belakang, Color::Named(NamedColor::Background));

                let tersembunyi = sel.flags.contains(Flags::HIDDEN);

                let belakang = warna_rgb(warna_belakang, warna_teks, warna_latar);

                let depan = if tersembunyi {
                    belakang
                } else {
                    warna_rgb(warna_depan, warna_teks, warna_latar)
                };

                if let Some(terakhir) = potongan.last_mut()
                    && terakhir.warna_teks == depan
                    && terakhir.warna_latar == belakang
                    && terakhir.latar_default == latar_default
                    && terakhir.tersembunyi == tersembunyi
                {
                    terakhir.teks.push_str(&teks);
                    continue;
                }

                potongan.push(PotonganWarna {
                    teks,
                    warna_teks: depan,
                    warna_latar: belakang,
                    latar_default,
                    tersembunyi,
                });
            }

            while let Some(terakhir) = potongan.last_mut() {
                let panjang = terakhir.teks.trim_end_matches(' ').len();
                terakhir.teks.truncate(panjang);

                if terakhir.teks.is_empty() {
                    potongan.pop();
                } else {
                    break;
                }
            }

            hasil.push(potongan);
        }

        hasil
    }
}
