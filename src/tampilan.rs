use std::time::Duration;

use gpui::{
    ClipboardItem, Context, CursorStyle, FocusHandle, Font, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, Pixels, Point, ScrollHandle, Subscription, Task, TextRun,
    Window, div, prelude::*, px, rgb, white,
};

use waifujahat_terminal::terminal::{
    LayarTerminal, MasukanTerminal, MotorTerminal, PesanTerminal, PosisiSeleksi, SeleksiTerminal,
    SesiTerminal,
};

use crate::tema::{WARNA_AKSEN, WARNA_LATAR, WARNA_TEKS};

const MAKSIMAL_RIWAYAT: usize = 100;
const WARNA_SELEKSI: u32 = 0x493663;

pub(crate) struct JendelaTerminal {
    fokus: FocusHandle,
    gulir: ScrollHandle,
    masukan: MasukanTerminal,
    layar: LayarTerminal,
    motor: MotorTerminal,
    sesi: Option<SesiTerminal>,
    status: String,
    riwayat_perintah: Vec<String>,
    indeks_riwayat: Option<usize>,
    draf_perintah: String,
    seleksi: SeleksiTerminal,
    ukuran_pty: (u16, u16),
    _pengamat_ukuran: Subscription,
    _tugas_pantau: Task<()>,
}

impl JendelaTerminal {
    pub(crate) fn baru(jendela: &mut Window, konteks: &mut Context<Self>) -> Self {
        let fokus = konteks.focus_handle().tab_stop(true);
        jendela.focus(&fokus, konteks);

        let (sesi, status) = match SesiTerminal::baru() {
            Ok(sesi) => (Some(sesi), String::new()),
            Err(kesalahan) => (None, format!("Gagal membuka PowerShell: {kesalahan}")),
        };

        let pengamat_ukuran =
            konteks.observe_window_bounds(jendela, |terminal, jendela, konteks| {
                terminal.sesuaikan_ukuran(jendela, konteks);
            });

        let tugas_pantau = konteks.spawn(async move |terminal, konteks| {
            loop {
                konteks
                    .background_executor()
                    .timer(Duration::from_millis(40))
                    .await;

                match terminal.update(konteks, |terminal, konteks| {
                    terminal.periksa_keluaran(konteks)
                }) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => break,
                }
            }
        });

        let mut terminal = Self {
            fokus,
            gulir: ScrollHandle::new(),
            masukan: MasukanTerminal::baru(),
            layar: LayarTerminal::baru(),
            motor: MotorTerminal::baru(80, 24),
            sesi,
            status,
            riwayat_perintah: Vec::new(),
            indeks_riwayat: None,
            draf_perintah: String::new(),
            seleksi: SeleksiTerminal::baru(),
            ukuran_pty: (80, 24),
            _pengamat_ukuran: pengamat_ukuran,
            _tugas_pantau: tugas_pantau,
        };

        terminal.sesuaikan_ukuran(jendela, konteks);
        terminal
    }

    fn sesuaikan_ukuran(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        let contoh = "MMMMMMMMMM";

        let run = TextRun {
            len: contoh.len(),
            font: Font {
                family: "Consolas".into(),
                ..Default::default()
            },
            color: white(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let bentuk = jendela
            .text_system()
            .shape_line(contoh.into(), px(14.0), &[run], None);

        let lebar_karakter = (f32::from(bentuk.width) / 10.0).max(1.0);
        let ukuran = jendela.viewport_size();

        let lebar_tersedia = (f32::from(ukuran.width) - 16.0).max(1.0);
        let tinggi_tersedia = (f32::from(ukuran.height) - 56.0).max(1.0);

        let kolom = (lebar_tersedia / lebar_karakter)
            .floor()
            .clamp(20.0, u16::MAX as f32) as u16;

        let baris = (tinggi_tersedia / 18.0).floor().clamp(3.0, u16::MAX as f32) as u16;

        if self.ukuran_pty == (kolom, baris) {
            return;
        }

        if let Some(sesi) = &self.sesi
            && let Err(kesalahan) = sesi.ubah_ukuran(kolom, baris)
        {
            self.status = format!("Gagal resize ConPTY: {kesalahan}");
            konteks.notify();
            return;
        }

        self.layar.ubah_tinggi(baris as usize);
        self.motor.ubah_ukuran(kolom as usize, baris as usize);
        self.ukuran_pty = (kolom, baris);

        konteks.notify();
    }

    fn periksa_keluaran(&mut self, konteks: &mut Context<Self>) -> bool {
        let mut berubah = false;

        for _ in 0..64 {
            let pesan = self.sesi.as_ref().and_then(SesiTerminal::try_ambil);

            match pesan {
                Some(PesanTerminal::Keluaran(buffer)) => {
                    self.layar.terima(&buffer);
                    self.motor.terima(&buffer);
                    berubah = true;
                }

                Some(PesanTerminal::Kesalahan(pesan)) => {
                    self.status = pesan;
                    self.sesi = None;
                    berubah = true;
                    break;
                }

                Some(PesanTerminal::Berakhir) => {
                    self.status = "Sesi PowerShell telah berakhir".into();
                    self.sesi = None;
                    berubah = true;
                    break;
                }

                None => break,
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

    fn baris_terminal(&self) -> Vec<String> {
        let mut baris = self.layar.baris();

        while baris.last().is_some_and(|teks| teks.is_empty()) {
            baris.pop();
        }

        let baris_terakhir = baris.pop().unwrap_or_default();

        baris.push(format!("{baris_terakhir}{}", self.masukan.teks()));
        baris
    }

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

    fn mouse_ditekan(
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

    fn mouse_digerakkan(
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

        let teks = teks
            .replace("\r\n", " ")
            .replace('\n', " ")
            .replace('\r', " ");

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

        if !perintah.trim().is_empty() {
            if self.riwayat_perintah.len() >= MAKSIMAL_RIWAYAT {
                self.riwayat_perintah.remove(0);
            }

            self.riwayat_perintah.push(perintah);
        }

        self.indeks_riwayat = None;
        self.draf_perintah.clear();
        self.seleksi.bersihkan();
        self.status.clear();
    }

    fn perintah_sebelumnya(&mut self) {
        if self.riwayat_perintah.is_empty() {
            return;
        }

        let indeks = match self.indeks_riwayat {
            Some(indeks) => indeks.saturating_sub(1),

            None => {
                self.draf_perintah = self.masukan.teks().to_string();
                self.riwayat_perintah.len() - 1
            }
        };

        self.indeks_riwayat = Some(indeks);
        self.masukan.ganti(self.riwayat_perintah[indeks].clone());
    }

    fn perintah_berikutnya(&mut self) {
        let Some(indeks) = self.indeks_riwayat else {
            return;
        };

        if indeks + 1 < self.riwayat_perintah.len() {
            let indeks_baru = indeks + 1;

            self.indeks_riwayat = Some(indeks_baru);
            self.masukan
                .ganti(self.riwayat_perintah[indeks_baru].clone());
        } else {
            self.indeks_riwayat = None;
            self.masukan.ganti(std::mem::take(&mut self.draf_perintah));
        }
    }

    fn selesai_navigasi_riwayat(&mut self) {
        self.indeks_riwayat = None;
        self.draf_perintah.clear();
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

    fn proses_tombol(
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

    fn buat_informasi_terminal(&self) -> impl IntoElement {
        div()
            .text_color(rgb(WARNA_TEKS))
            .child("Waifujahat Terminal 0.1.0")
    }

    fn buat_keluaran_terminal(&self, konteks: &mut Context<Self>) -> impl IntoElement {
        let baris = self.baris_terminal();
        let jumlah_baris = baris.len();

        let baris_motor = self.motor.baris_layar();
        let warna_motor = self.motor.baris_berwarna(WARNA_TEKS, WARNA_LATAR);
        let pergeseran = jumlah_baris.saturating_sub(baris_motor.len());

        let awalan = self
            .layar
            .baris()
            .into_iter()
            .rev()
            .find(|teks| !teks.is_empty())
            .unwrap_or_default();

        let mut keluaran = div()
            .id("keluaran-terminal")
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.gulir)
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, kejadian, jendela, konteks| {
                    terminal.mouse_ditekan(kejadian, jendela, konteks);
                    konteks.stop_propagation();
                }),
            );

        for (indeks, teks) in baris.into_iter().enumerate() {
            let terakhir = indeks + 1 == jumlah_baris;

            let tampilan = if terakhir {
                self.masukan.tampilkan(&awalan)
            } else if teks.is_empty() {
                " ".to_string()
            } else {
                teks.clone()
            };

            let mut elemen_baris = div()
                .id(format!("baris-terminal-{indeks}"))
                .w_full()
                .h(px(18.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .whitespace_nowrap();

            if let Some((mulai, selesai)) = self.seleksi.potongan_baris(indeks, &teks) {
                let karakter: Vec<char> = teks.chars().collect();

                let sebelum: String = karakter[..mulai].iter().collect();
                let dipilih: String = karakter[mulai..selesai].iter().collect();
                let sesudah: String = karakter[selesai..].iter().collect();

                elemen_baris = elemen_baris
                    .child(div().whitespace_nowrap().child(sebelum))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .bg(rgb(WARNA_SELEKSI))
                            .child(dipilih),
                    )
                    .child(div().whitespace_nowrap().child(sesudah));
            } else if !terakhir {
                let indeks_motor = indeks.checked_sub(pergeseran);

                let baris_cocok = indeks_motor
                    .and_then(|indeks| baris_motor.get(indeks).zip(warna_motor.get(indeks)))
                    .filter(|(teks_motor, _)| teks_motor.trim_end() == teks.trim_end());

                if let Some((_, potongan)) = baris_cocok {
                    let mut teks_berwarna = div().flex().items_center().whitespace_nowrap();

                    for bagian in potongan {
                        teks_berwarna = teks_berwarna.child(
                            div()
                                .whitespace_nowrap()
                                .text_color(rgb(bagian.warna_teks))
                                .bg(rgb(bagian.warna_latar))
                                .child(bagian.teks.clone()),
                        );
                    }

                    elemen_baris = elemen_baris.child(teks_berwarna);
                } else {
                    elemen_baris = elemen_baris.child(div().whitespace_nowrap().child(tampilan));
                }
            } else {
                elemen_baris = elemen_baris.child(div().whitespace_nowrap().child(tampilan));
            }

            keluaran = keluaran.child(elemen_baris);
        }

        keluaran
    }
}

impl Render for JendelaTerminal {
    fn render(&mut self, _jendela: &mut Window, konteks: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("jendela-terminal")
            .track_focus(&self.fokus)
            .on_key_down(konteks.listener(Self::proses_tombol))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, jendela, konteks| {
                    jendela.focus(&terminal.fokus, konteks);
                    terminal.seleksi.bersihkan();
                    konteks.notify();
                }),
            )
            .on_mouse_move(konteks.listener(|terminal, kejadian, jendela, konteks| {
                terminal.mouse_digerakkan(kejadian, jendela, konteks);
            }))
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, _| {
                    terminal.seleksi.selesai();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, _| {
                    terminal.seleksi.selesai();
                }),
            )
            .size_full()
            .flex()
            .flex_col()
            .p_2()
            .gap_1()
            .bg(rgb(WARNA_LATAR))
            .text_color(rgb(WARNA_TEKS))
            .font_family("Consolas")
            .text_size(px(14.0))
            .line_height(px(18.0))
            .child(self.buat_informasi_terminal())
            .child(self.buat_keluaran_terminal(konteks))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(WARNA_AKSEN))
                    .child(self.status.clone()),
            )
    }
}
