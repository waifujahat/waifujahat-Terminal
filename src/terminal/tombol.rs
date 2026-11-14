/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Keyboard input
 * Purpose: Translates keyboard input into terminal control sequences.
 */

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
    Awal,
    Akhir,
    Kontrol(char),
}

impl TombolTerminal<'_> {
    pub fn ke_bytes(self) -> Option<Vec<u8>> {
        let kode: &[u8] = match self {
            Self::Teks(teks) => {
                return (!teks.is_empty()).then(|| teks.as_bytes().to_vec());
            }
            Self::Enter => b"\r",
            Self::Tab => b"\t",
            Self::TabMundur => b"\x1b[Z",
            Self::HapusSebelum => b"\x7f",
            Self::HapusSesudah => b"\x1b[3~",
            Self::Escape => b"\x1b",
            Self::Atas => b"\x1b[A",
            Self::Bawah => b"\x1b[B",
            Self::Kiri => b"\x1b[D",
            Self::Kanan => b"\x1b[C",
            Self::Awal => b"\x1b[H",
            Self::Akhir => b"\x1b[F",
            Self::Kontrol(karakter) => {
                if karakter.is_ascii_alphabetic() {
                    let kode = karakter.to_ascii_uppercase() as u8 - b'@';

                    return Some(vec![kode]);
                }

                return None;
            }
        };
        Some(kode.to_vec())
    }
}
