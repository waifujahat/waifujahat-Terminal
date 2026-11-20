/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Win32 keyboard
 * Purpose: Encodes native Windows keyboard records
 */

use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RekamanTombolWin32 {
    pub kode_virtual: u16,
    pub kode_pindai: u16,
    pub unicode: u16,
    pub ditekan: bool,
    pub status_kontrol: u32,
    pub jumlah_pengulangan: NonZeroU16,
}

impl RekamanTombolWin32 {
    pub fn sandi(&self) -> Vec<u8> {
        format!(
            "\x1b[{};{};{};{};{};{}_",
            self.kode_virtual,
            self.kode_pindai,
            self.unicode,
            u8::from(self.ditekan),
            self.status_kontrol,
            self.jumlah_pengulangan,
        )
        .into_bytes()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetadataTombolWin32 {
    pub kode_virtual: u16,
    pub kode_pindai: u16,
    pub ditekan: bool,
    pub ditekan_sebelumnya: bool,
    pub diperluas: bool,
    pub konteks_alt: bool,
    pub jumlah_pengulangan: NonZeroU16,
}

impl MetadataTombolWin32 {
    pub fn dari_pesan(kode_virtual: u16, parameter: u32) -> Option<Self> {
        let jumlah_pengulangan = NonZeroU16::new(parameter as u16)?;

        Some(Self {
            kode_virtual,
            kode_pindai: ((parameter >> 16) & 0xff) as u16,
            ditekan: parameter & (1 << 31) == 0,
            ditekan_sebelumnya: parameter & (1 << 30) != 0,
            diperluas: parameter & (1 << 24) != 0,
            konteks_alt: parameter & (1 << 29) != 0,
            jumlah_pengulangan,
        })
    }
}

impl MetadataTombolWin32 {
    pub fn status_kontrol(&self, keadaan: &[u8; 256]) -> u32 {
        let mut status = if self.diperluas { 0x0100 } else { 0 };

        for (kode, flag) in [
            (0xA5, 0x0001), // Right Alt
            (0xA4, 0x0002), // Left Alt
            (0xA3, 0x0004), // Right Ctrl
            (0xA2, 0x0008), // Left Ctrl
            (0x10, 0x0010), // Shift
        ] {
            if keadaan[kode] & 0x80 != 0 {
                status |= flag;
            }
        }

        for (kode, flag) in [
            (0x90, 0x0020), // Num Lock
            (0x91, 0x0040), // Scroll Lock
            (0x14, 0x0080), // Caps Lock
        ] {
            if keadaan[kode] & 0x01 != 0 {
                status |= flag;
            }
        }

        status
    }

    pub fn ke_rekaman(&self, unicode: u16, keadaan: &[u8; 256]) -> RekamanTombolWin32 {
        RekamanTombolWin32 {
            kode_virtual: self.kode_virtual,
            kode_pindai: self.kode_pindai,
            unicode,
            ditekan: self.ditekan,
            status_kontrol: self.status_kontrol(keadaan),
            jumlah_pengulangan: self.jumlah_pengulangan,
        }
    }
}
