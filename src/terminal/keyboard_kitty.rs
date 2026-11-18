/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Kitty keyboard
 * Purpose: Encodes Unicode keyboard events using CSI-u sequences
 */

use super::tombol::{PengubahTombol, TombolKhusus, TombolTerminal};

pub fn sandi_kitty_tombol(
    tombol: TombolTerminal<'_>,
    aksi: JenisAksiTombol,
    laporkan_aksi: bool,
) -> Option<Vec<u8>> {
    let normal = PengubahTombol::default();

    let shift = PengubahTombol {
        shift: true,
        ..normal
    };

    let (tombol_khusus, pengubah) = match tombol {
        TombolTerminal::Teks(_)
        | TombolTerminal::Enter
        | TombolTerminal::Tab
        | TombolTerminal::TabMundur
        | TombolTerminal::HapusSebelum => {
            if aksi == JenisAksiTombol::Lepas {
                return None;
            }

            return tombol.ke_bytes();
        }

        TombolTerminal::Escape => {
            return sandi_kitty_unicode('\x1b', normal, aksi, laporkan_aksi);
        }

        TombolTerminal::Kontrol(karakter) => {
            if !karakter.is_ascii_alphabetic() {
                return None;
            }

            return sandi_kitty_unicode(
                karakter.to_ascii_lowercase(),
                PengubahTombol {
                    kontrol: true,
                    ..normal
                },
                aksi,
                laporkan_aksi,
            );
        }

        TombolTerminal::Atas => (TombolKhusus::Atas, normal),
        TombolTerminal::Bawah => (TombolKhusus::Bawah, normal),
        TombolTerminal::Kiri => (TombolKhusus::Kiri, normal),
        TombolTerminal::Kanan => (TombolKhusus::Kanan, normal),
        TombolTerminal::Awal => (TombolKhusus::Awal, normal),
        TombolTerminal::Akhir => (TombolKhusus::Akhir, normal),
        TombolTerminal::HapusSesudah => (TombolKhusus::HapusSesudah, normal),

        TombolTerminal::PilihKiri => (TombolKhusus::Kiri, shift),
        TombolTerminal::PilihKanan => (TombolKhusus::Kanan, shift),

        TombolTerminal::Khusus(tombol, pengubah) => (tombol, pengubah),
    };

    sandi_kitty_khusus(tombol_khusus, pengubah, aksi, laporkan_aksi)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JenisAksiTombol {
    Tekan,
    Ulang,
    Lepas,
}

pub fn sandi_kitty_unicode(
    karakter: char,
    pengubah: PengubahTombol,
    aksi: JenisAksiTombol,
    laporkan_aksi: bool,
) -> Option<Vec<u8>> {
    sandi_kitty_csi(u32::from(karakter), 'u', pengubah, aksi, laporkan_aksi)
}

pub fn sandi_kitty_khusus(
    tombol: TombolKhusus,
    pengubah: PengubahTombol,
    aksi: JenisAksiTombol,
    laporkan_aksi: bool,
) -> Option<Vec<u8>> {
    let (kode, akhiran) = match tombol {
        TombolKhusus::Atas => (1, 'A'),
        TombolKhusus::Bawah => (1, 'B'),
        TombolKhusus::Kanan => (1, 'C'),
        TombolKhusus::Kiri => (1, 'D'),
        TombolKhusus::Awal => (1, 'H'),
        TombolKhusus::Akhir => (1, 'F'),
        TombolKhusus::Sisipkan => (2, '~'),
        TombolKhusus::HapusSesudah => (3, '~'),
        TombolKhusus::HalamanAtas => (5, '~'),
        TombolKhusus::HalamanBawah => (6, '~'),

        TombolKhusus::Fungsi(nomor) => match nomor {
            1 => (1, 'P'),
            2 => (1, 'Q'),
            3 => (13, '~'),
            4 => (1, 'S'),
            5 => (15, '~'),
            6 => (17, '~'),
            7 => (18, '~'),
            8 => (19, '~'),
            9 => (20, '~'),
            10 => (21, '~'),
            11 => (23, '~'),
            12 => (24, '~'),
            13..=35 => (57376 + u32::from(nomor - 13), 'u'),
            _ => return None,
        },
    };

    sandi_kitty_csi(kode, akhiran, pengubah, aksi, laporkan_aksi)
}

fn sandi_kitty_csi(
    kode: u32,
    akhiran: char,
    pengubah: PengubahTombol,
    aksi: JenisAksiTombol,
    laporkan_aksi: bool,
) -> Option<Vec<u8>> {
    if aksi == JenisAksiTombol::Lepas && !laporkan_aksi {
        return None;
    }

    let parameter = pengubah.parameter_xterm();

    let hasil = match (laporkan_aksi, aksi) {
        (true, JenisAksiTombol::Ulang) => {
            format!("\x1b[{kode};{parameter}:2{akhiran}")
        }

        (true, JenisAksiTombol::Lepas) => {
            format!("\x1b[{kode};{parameter}:3{akhiran}")
        }

        _ if parameter == 1 && kode == 1 && !matches!(akhiran, 'u' | '~') => {
            format!("\x1b[{akhiran}")
        }

        _ if parameter == 1 => {
            format!("\x1b[{kode}{akhiran}")
        }

        _ => {
            format!("\x1b[{kode};{parameter}{akhiran}")
        }
    };

    Some(hasil.into_bytes())
}
