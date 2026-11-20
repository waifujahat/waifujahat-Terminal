/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Win32 input
 * Purpose: Routes native keyboard records to their owning terminal session
 */

use waifujahat_terminal::terminal::{ProtokolKeyboard, RekamanTombolWin32};

use super::ruang_sesi::RuangSesi;

use std::num::NonZeroU16;

pub(super) fn identitas_tombol_win32(rekaman: RekamanTombolWin32) -> String {
    format!(
        "win32:{}:{}:{}",
        rekaman.kode_virtual,
        rekaman.kode_pindai,
        rekaman.status_kontrol & 0x0100
    )
}

fn identitas_sama(awal: &RekamanTombolWin32, akhir: &RekamanTombolWin32) -> bool {
    awal.kode_virtual == akhir.kode_virtual
        && awal.kode_pindai == akhir.kode_pindai
        && (awal.status_kontrol & 0x0100) == (akhir.status_kontrol & 0x0100)
}

impl RuangSesi {
    fn kirim_rekaman_win32(&mut self, rekaman: RekamanTombolWin32) -> bool {
        let Some(data) = self.keadaan.motor.sandi_rekaman_win32(rekaman) else {
            return false;
        };

        let Some(sesi) = &self.sesi else {
            return false;
        };

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal mengirim input Win32: {kesalahan}");

            return false;
        }

        true
    }

    pub(super) fn tekan_tombol_win32(&mut self, rekaman: RekamanTombolWin32) -> bool {
        if !rekaman.ditekan
            || self.keadaan.motor.protokol_keyboard() != ProtokolKeyboard::Win32
            || self.sesi.is_none()
        {
            return false;
        }

        if self.kirim_rekaman_win32(rekaman) {
            let identitas = identitas_tombol_win32(rekaman);

            self.tombol_win32_ditekan.insert(identitas, rekaman);
        }

        true
    }

    pub(super) fn ulangi_tombol_win32(
        &mut self,
        nama: &str,
        rekaman: Option<RekamanTombolWin32>,
    ) -> bool {
        let Some(sebelumnya) = self.tombol_win32_ditekan.get(nama).copied() else {
            return false;
        };

        if let Some(rekaman) = rekaman
            && rekaman.ditekan
            && identitas_sama(&sebelumnya, &rekaman)
            && self.kirim_rekaman_win32(rekaman)
        {
            self.tombol_win32_ditekan.insert(nama.to_owned(), rekaman);
        }

        true
    }

    pub(super) fn lepas_tombol_win32(
        &mut self,
        nama: &str,
        rekaman: Option<RekamanTombolWin32>,
    ) -> bool {
        let Some(sebelumnya) = self.tombol_win32_ditekan.remove(nama) else {
            return false;
        };

        let rekaman = rekaman
            .filter(|rekaman| !rekaman.ditekan && identitas_sama(&sebelumnya, rekaman))
            .unwrap_or(RekamanTombolWin32 {
                ditekan: false,
                jumlah_pengulangan: NonZeroU16::MIN,
                ..sebelumnya
            });

        self.kirim_rekaman_win32(rekaman);

        true
    }

    pub(super) fn lepas_semua_tombol_win32(&mut self) {
        let daftar = std::mem::take(&mut self.tombol_win32_ditekan);

        for rekaman in daftar.into_values() {
            self.kirim_rekaman_win32(RekamanTombolWin32 {
                ditekan: false,
                jumlah_pengulangan: NonZeroU16::MIN,
                ..rekaman
            });
        }
    }
}
