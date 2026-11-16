/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Terminal sizing
 * Purpose: Calculates PTY dimensions from individual panel bounds
 */

use gpui::{Font, TextRun, Window, px, white};

use super::ruang_sesi::RuangSesi;

impl RuangSesi {
    pub(super) fn lebar_karakter_grid(jendela: &mut Window) -> f32 {
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

        (f32::from(bentuk.width) / 10.0).round().max(1.0)
    }

    pub(super) fn sesuaikan_ukuran_panel(&mut self, lebar_karakter: f32) -> bool {
        let batas = self.gulir.bounds();

        let lebar = f32::from(batas.size.width);
        let tinggi = f32::from(batas.size.height);

        if lebar <= 0.0 || tinggi <= 0.0 {
            return false;
        }

        let kolom = (lebar / lebar_karakter)
            .floor()
            .clamp(20.0, u16::MAX as f32) as u16;

        let baris = (tinggi / 18.0).floor().clamp(3.0, u16::MAX as f32) as u16;

        let ukuran = (kolom, baris);

        if self.ukuran_terakhir_diminta == Some(ukuran) {
            return false;
        }

        self.ukuran_terakhir_diminta = Some(ukuran);

        if self.ukuran_pty == ukuran {
            return false;
        }

        if let Some(sesi) = &self.sesi
            && let Err(kesalahan) = sesi.ubah_ukuran(kolom, baris)
        {
            self.status = format!("Gagal resize ConPTY: {kesalahan}");

            return true;
        }

        self.keadaan.ubah_ukuran(kolom as usize, baris as usize);

        self.ukuran_pty = ukuran;

        true
    }
}
