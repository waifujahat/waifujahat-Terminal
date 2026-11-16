/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Session state
 * Purpose: Owns isolated terminal state for each session
 */

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
}

impl KeadaanSesi {
    pub fn baru(kolom: usize, baris: usize) -> Self {
        let mut layar = LayarTerminal::baru();

        layar.ubah_tinggi(baris);

        Self {
            layar,
            motor: MotorTerminal::baru(kolom, baris),
            siklus: SiklusPerintah::baru(),
            seleksi: SeleksiTerminal::baru(),
            integrasi_shell: ParserIntegrasiShell::baru(),
        }
    }

    pub fn terima(&mut self, keluaran: &[u8]) -> Vec<PenandaShell> {
        self.layar.terima(keluaran);
        self.motor.terima(keluaran);

        self.integrasi_shell.terima(keluaran)
    }

    pub fn ubah_ukuran(&mut self, kolom: usize, baris: usize) {
        self.layar.ubah_tinggi(baris);
        self.motor.ubah_ukuran(kolom, baris);
    }
}
