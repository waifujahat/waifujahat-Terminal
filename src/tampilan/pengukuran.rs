/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Terminal sizing
 * Purpose: Calculates PTY cell dimensions from the window viewport.
 */

use gpui::{Context, Font, TextRun, Window, px, white};

use super::JendelaTerminal;

impl JendelaTerminal {
    pub(super) fn sesuaikan_ukuran(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
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
}
