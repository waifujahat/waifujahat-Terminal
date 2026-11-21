/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Terminal sizing
 * Purpose : Calculates PTY dimensions from configured font metrics
 */

use gpui::{Font, TextRun, Window, px, white};

use super::ruang_sesi::RuangSesi;

impl RuangSesi {
    pub(super) fn lebar_karakter_grid(&self, jendela: &mut Window) -> f32 {
        let contoh = "MMMMMMMMMM";

        let run = TextRun {
            len: contoh.len(),
            font: Font {
                family: self.pengaturan.keluarga_font.clone().into(),
                ..Default::default()
            },
            color: white(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let bentuk = jendela.text_system().shape_line(
            contoh.into(),
            px(self.pengaturan.ukuran_font_efektif()),
            &[run],
            None,
        );

        (f32::from(bentuk.width) / 10.0).round().max(1.0)
    }

    pub(super) fn sesuaikan_ukuran_panel(&mut self, lebar_karakter: f32) -> bool {
        let batas = self.gulir.bounds();

        let lebar = f32::from(batas.size.width);
        let tinggi = f32::from(batas.size.height);

        let Some(ukuran) = self.pengaturan.ukuran_grid(lebar, tinggi, lebar_karakter) else {
            return false;
        };

        let (kolom, baris) = ukuran;

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
            self.status = format!("Gagal resize ConPTY : {kesalahan}");

            return true;
        }

        self.keadaan.ubah_ukuran(kolom as usize, baris as usize);

        self.ukuran_pty = ukuran;

        true
    }
}
