/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / Grid geometry
 * Purpose : Synchronizes terminal dimensions and pointer coordinates
 */

use super::PengaturanTerminal;

impl PengaturanTerminal {
    pub fn ukuran_grid(
        &self,
        lebar_panel: f32,
        tinggi_panel: f32,
        lebar_karakter: f32,
    ) -> Option<(u16, u16)> {
        let tinggi_baris = self.tinggi_baris();

        if ![lebar_panel, tinggi_panel, lebar_karakter, tinggi_baris]
            .into_iter()
            .all(|nilai| nilai.is_finite() && nilai > 0.0)
        {
            return None;
        }

        let kolom = (lebar_panel / lebar_karakter)
            .floor()
            .clamp(20.0, u16::MAX as f32) as u16;

        let baris = (tinggi_panel / tinggi_baris)
            .floor()
            .clamp(3.0, u16::MAX as f32) as u16;

        Some((kolom, baris))
    }

    pub fn posisi_grid(&self, x: f32, y: f32, lebar_karakter: f32) -> Option<(usize, usize)> {
        let tinggi_baris = self.tinggi_baris();

        if !x.is_finite()
            || !y.is_finite()
            || !lebar_karakter.is_finite()
            || !tinggi_baris.is_finite()
            || x < 0.0
            || y < 0.0
            || lebar_karakter <= 0.0
            || tinggi_baris <= 0.0
        {
            return None;
        }

        let kolom = (x / lebar_karakter).floor() as usize;

        let baris = (y / tinggi_baris).floor() as usize;

        Some((kolom, baris))
    }

    pub fn langkah_gulir_grid(&self, jarak: f32) -> isize {
        let tinggi_baris = self.tinggi_baris();

        if !jarak.is_finite() || jarak == 0.0 || !tinggi_baris.is_finite() || tinggi_baris <= 0.0 {
            return 0;
        }

        let jumlah = (jarak / tinggi_baris).round() as isize;

        if jumlah == 0 {
            jarak.signum() as isize
        } else {
            jumlah
        }
    }
}
