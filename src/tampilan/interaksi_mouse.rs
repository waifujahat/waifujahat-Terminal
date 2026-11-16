/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Mouse reporting
 * Purpose: Forwards mouse input to interactive terminal applications
 */

use gpui::{Context, Modifiers, MouseDownEvent, MouseUpEvent, Pixels, Point, Window};
use waifujahat_terminal::terminal::{AksiMouse, PengubahMouse, TombolMouse, sandi_mouse_sgr};

use super::JendelaTerminal;

impl JendelaTerminal {
    fn koordinat_mouse_grid(
        &self,
        posisi: Point<Pixels>,
        jendela: &mut Window,
    ) -> Option<(usize, usize)> {
        let batas = self.gulir.bounds_for_item(0)?;
        let offset = self.gulir.offset();

        let lebar_karakter = Self::lebar_karakter_grid(jendela);

        let x = f32::from(posisi.x - (batas.left() + offset.x));
        let y = f32::from(posisi.y - (batas.top() + offset.y));

        if x < 0.0 || y < 0.0 {
            return None;
        }

        let kolom = (x / lebar_karakter).floor() as usize;
        let baris = (y / 18.0).floor() as usize;

        let (jumlah_kolom, jumlah_baris) = self.motor.ukuran();

        (kolom < jumlah_kolom && baris < jumlah_baris).then_some((kolom, baris))
    }

    fn kirim_mouse_tui(&mut self, aksi: AksiMouse, posisi: (usize, usize), pengubah: &Modifiers) {
        let Some(sesi) = &self.sesi else {
            self.status = "Sesi PowerShell tidak tersedia".into();
            return;
        };

        let data = sandi_mouse_sgr(
            aksi,
            posisi.0,
            posisi.1,
            PengubahMouse {
                shift: pengubah.shift,
                alt: pengubah.alt,
                kontrol: pengubah.control,
            },
        );

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal mengirim input mouse: {kesalahan}");
        }
    }

    pub(super) fn tangani_tekan_mouse_tui(
        &mut self,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) -> bool {
        if !self.mode_grid_aktif()
            || !self.motor.pelaporan_mouse_aktif()
            || !self.motor.mouse_sgr_aktif()
            || kejadian.modifiers.shift
            || self.sesi.is_none()
        {
            return false;
        }

        let Some(posisi) = self.koordinat_mouse_grid(kejadian.position, jendela) else {
            return false;
        };

        self.kirim_mouse_tui(
            AksiMouse::Tekan(TombolMouse::Kiri),
            posisi,
            &kejadian.modifiers,
        );

        self.mouse_tui_ditekan = Some(posisi);
        self.seleksi.bersihkan();
        self.arah_gulir_seleksi = 0;

        konteks.notify();
        true
    }

    pub(super) fn tangani_lepas_mouse_tui(
        &mut self,
        kejadian: &MouseUpEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if let Some(posisi_sebelum) = self.mouse_tui_ditekan.take() {
            let posisi = self
                .koordinat_mouse_grid(kejadian.position, jendela)
                .unwrap_or(posisi_sebelum);

            if self.motor.pelaporan_mouse_aktif() && self.motor.mouse_sgr_aktif() {
                self.kirim_mouse_tui(
                    AksiMouse::Lepas(TombolMouse::Kiri),
                    posisi,
                    &kejadian.modifiers,
                );
            }
        } else if let Some(posisi) = self.seleksi.posisi_klik() {
            self.pindah_kursor_mouse(posisi);
        }

        self.arah_gulir_seleksi = 0;
        self.seleksi.selesai();
        konteks.notify();
    }
}
