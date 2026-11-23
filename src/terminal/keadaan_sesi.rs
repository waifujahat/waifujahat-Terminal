/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Session state
 * Purpose : Owns isolated state for local and remote terminal sessions
 */

use std::path::Path;

use super::{
    LayarTerminal, MotorTerminal, ParserIntegrasiShell, PenandaShell, SeleksiTerminal,
    SiklusPerintah,
};

pub struct KeadaanSesi {
    pub layar: LayarTerminal,
    pub motor: MotorTerminal,
    pub siklus: SiklusPerintah,
    pub seleksi: SeleksiTerminal,

    integrasi_shell: ParserIntegrasiShell,
    integrasi_shell_aktif: bool,
    integrasi_shell_diizinkan: bool,
}

impl KeadaanSesi {
    pub fn baru(kolom: usize, baris: usize) -> Self {
        Self::baru_dengan_pengaturan(kolom, baris, true, None)
    }

    pub fn baru_dengan_pengaturan(
        kolom: usize,
        baris: usize,
        konfirmasi_aktif: bool,
        batas_riwayat: Option<usize>,
    ) -> Self {
        Self::buat(kolom, baris, konfirmasi_aktif, batas_riwayat, true)
    }

    pub fn baru_untuk_ssh(kolom: usize, baris: usize, batas_riwayat: Option<usize>) -> Self {
        Self::buat(kolom, baris, false, batas_riwayat, false)
    }

    fn buat(
        kolom: usize,
        baris: usize,
        konfirmasi_aktif: bool,
        batas_riwayat: Option<usize>,
        integrasi_shell_diizinkan: bool,
    ) -> Self {
        let mut layar = LayarTerminal::baru();

        layar.ubah_tinggi(baris);

        Self {
            layar,
            motor: MotorTerminal::baru_dengan_kitty_dan_riwayat(kolom, baris, batas_riwayat),
            siklus: SiklusPerintah::baru_dengan_konfirmasi(konfirmasi_aktif),
            seleksi: SeleksiTerminal::baru(),
            integrasi_shell: ParserIntegrasiShell::baru(),
            integrasi_shell_aktif: false,
            integrasi_shell_diizinkan,
        }
    }

    pub fn terima(&mut self, keluaran: &[u8]) -> Vec<PenandaShell> {
        self.layar.terima(keluaran);
        self.motor.terima(keluaran);

        if !self.integrasi_shell_diizinkan {
            return Vec::new();
        }

        let penanda = self.integrasi_shell.terima(keluaran);

        if !penanda.is_empty() {
            self.integrasi_shell_aktif = true;
        }

        penanda
    }

    pub fn integrasi_shell_aktif(&self) -> bool {
        self.integrasi_shell_aktif
    }

    pub fn direktori_terakhir(&self) -> Option<&Path> {
        self.integrasi_shell.direktori_terakhir()
    }

    pub fn perintah_selesai_dari_prompt(&self) -> bool {
        self.integrasi_shell_diizinkan
            && !self.integrasi_shell_aktif()
            && self.siklus.berjalan()
            && !self.motor.layar_alternatif_aktif()
            && self.prompt_powershell_terlihat()
    }

    fn prompt_powershell_terlihat(&self) -> bool {
        let Some(indeks_baris_awal) = self.siklus.indeks_baris_awal() else {
            return false;
        };

        let baris = self.layar.baris();

        let Some((indeks, teks)) = baris
            .iter()
            .enumerate()
            .rev()
            .find(|(_, teks)| !teks.trim().is_empty())
        else {
            return false;
        };

        let teks = teks.trim();

        indeks > indeks_baris_awal && teks.starts_with("PS ") && teks.ends_with('>')
    }

    pub fn ubah_ukuran(&mut self, kolom: usize, baris: usize) {
        self.layar.ubah_tinggi(baris);
        self.motor.ubah_ukuran(kolom, baris);
    }
}
