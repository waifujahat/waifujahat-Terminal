/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Keyboard input
 * Purpose: Encodes keyboard input using Xterm control sequences
 */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PengubahTombol {
    pub shift: bool,
    pub alt: bool,
    pub kontrol: bool,
}

impl PengubahTombol {
    pub(super) fn parameter_xterm(self) -> u8 {
        1 + u8::from(self.shift) + 2 * u8::from(self.alt) + 4 * u8::from(self.kontrol)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TombolKhusus {
    Atas,
    Bawah,
    Kiri,
    Kanan,
    Awal,
    Akhir,
    Sisipkan,
    HapusSesudah,
    HalamanAtas,
    HalamanBawah,
    Fungsi(u8),
}

impl TombolKhusus {
    fn sandi(self, pengubah: PengubahTombol, kursor_aplikasi: bool) -> Option<Vec<u8>> {
        let parameter = pengubah.parameter_xterm();

        let (akhiran, mode_aplikasi) = match self {
            Self::Atas => ('A', kursor_aplikasi),
            Self::Bawah => ('B', kursor_aplikasi),
            Self::Kiri => ('D', kursor_aplikasi),
            Self::Kanan => ('C', kursor_aplikasi),
            Self::Awal => ('H', kursor_aplikasi),
            Self::Akhir => ('F', kursor_aplikasi),

            Self::Sisipkan => {
                return Some(Self::sandi_tilde(2, parameter));
            }

            Self::HapusSesudah => {
                return Some(Self::sandi_tilde(3, parameter));
            }

            Self::HalamanAtas => {
                return Some(Self::sandi_tilde(5, parameter));
            }

            Self::HalamanBawah => {
                return Some(Self::sandi_tilde(6, parameter));
            }

            Self::Fungsi(nomor @ 1..=4) => (char::from(b'P' + nomor - 1), true),

            Self::Fungsi(nomor) => {
                let kode = match nomor {
                    5 => 15,
                    6 => 17,
                    7 => 18,
                    8 => 19,
                    9 => 20,
                    10 => 21,
                    11 => 23,
                    12 => 24,
                    _ => return None,
                };

                return Some(Self::sandi_tilde(kode, parameter));
            }
        };

        let hasil = if parameter != 1 {
            format!("\x1b[1;{parameter}{akhiran}")
        } else if mode_aplikasi {
            format!("\x1bO{akhiran}")
        } else {
            format!("\x1b[{akhiran}")
        };

        Some(hasil.into_bytes())
    }

    fn sandi_tilde(kode: u8, parameter: u8) -> Vec<u8> {
        if parameter == 1 {
            format!("\x1b[{kode}~").into_bytes()
        } else {
            format!("\x1b[{kode};{parameter}~").into_bytes()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TombolTerminal<'a> {
    Teks(&'a str),
    Enter,
    Tab,
    TabMundur,
    HapusSebelum,
    HapusSesudah,
    Escape,
    Atas,
    Bawah,
    Kiri,
    Kanan,
    PilihKiri,
    PilihKanan,
    Awal,
    Akhir,
    Kontrol(char),
    Khusus(TombolKhusus, PengubahTombol),
}

impl TombolTerminal<'_> {
    pub fn ke_bytes(self) -> Option<Vec<u8>> {
        self.ke_bytes_dengan_kursor_aplikasi(false)
    }

    pub fn ke_bytes_dengan_kursor_aplikasi(self, kursor_aplikasi: bool) -> Option<Vec<u8>> {
        let normal = PengubahTombol::default();

        let shift = PengubahTombol {
            shift: true,
            ..normal
        };

        let (tombol, pengubah) = match self {
            Self::Teks(teks) => {
                return (!teks.is_empty()).then(|| teks.as_bytes().to_vec());
            }

            Self::Enter => return Some(vec![b'\r']),
            Self::Tab => return Some(vec![b'\t']),
            Self::TabMundur => return Some(b"\x1b[Z".to_vec()),
            Self::HapusSebelum => return Some(vec![0x7f]),
            Self::Escape => return Some(vec![0x1b]),

            Self::Atas => (TombolKhusus::Atas, normal),
            Self::Bawah => (TombolKhusus::Bawah, normal),
            Self::Kiri => (TombolKhusus::Kiri, normal),
            Self::Kanan => (TombolKhusus::Kanan, normal),
            Self::Awal => (TombolKhusus::Awal, normal),
            Self::Akhir => (TombolKhusus::Akhir, normal),
            Self::HapusSesudah => (TombolKhusus::HapusSesudah, normal),

            Self::PilihKiri => (TombolKhusus::Kiri, shift),
            Self::PilihKanan => (TombolKhusus::Kanan, shift),

            Self::Khusus(tombol, pengubah) => (tombol, pengubah),

            Self::Kontrol(karakter) => {
                if karakter.is_ascii_alphabetic() {
                    let kode = karakter.to_ascii_uppercase() as u8 - b'@';

                    return Some(vec![kode]);
                }

                return None;
            }
        };

        tombol.sandi(pengubah, kursor_aplikasi)
    }
}
