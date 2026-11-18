/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Kitty protocol parser
 * Purpose: Parses keyboard protocol negotiation sequences
 */

use vte::{Params, Parser, Perform};

use super::mode_kitty::PengaturanModeKitty;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerintahModeKitty {
    Tanya,
    Dorong(u16),
    Keluarkan(usize),
    Atur(u16, PengaturanModeKitty),
    Reset,
    PeralihanLayar,
}

#[derive(Default)]
struct PenerimaPerintah {
    perintah: Option<PerintahModeKitty>,
}

impl Perform for PenerimaPerintah {
    fn csi_dispatch(&mut self, parameter: &Params, perantara: &[u8], abaikan: bool, aksi: char) {
        if abaikan {
            return;
        }

        if matches!(aksi, 'h' | 'l')
            && perantara == b"?"
            && parameter
                .iter()
                .any(|nilai| matches!(nilai, [47 | 1047 | 1049]))
        {
            self.perintah = Some(PerintahModeKitty::PeralihanLayar);
            return;
        }

        if aksi != 'u' {
            return;
        }

        let mut parameter = parameter.iter();

        let pertama = match parameter.next() {
            Some([nilai]) => *nilai,
            None => 0,
            _ => return,
        };

        let kedua = match parameter.next() {
            Some([nilai]) => Some(*nilai),
            None => None,
            _ => return,
        };

        if parameter.next().is_some() {
            return;
        }

        self.perintah = match perantara {
            b"?" if pertama == 0 && kedua.is_none() => Some(PerintahModeKitty::Tanya),

            b">" if kedua.is_none() => Some(PerintahModeKitty::Dorong(pertama)),

            b"<" if kedua.is_none() => {
                Some(PerintahModeKitty::Keluarkan(usize::from(pertama.max(1))))
            }

            b"=" => {
                let pengaturan = match kedua.unwrap_or(1) {
                    0 | 1 => PengaturanModeKitty::Ganti,
                    2 => PengaturanModeKitty::Tambah,
                    3 => PengaturanModeKitty::Hapus,
                    _ => return,
                };

                Some(PerintahModeKitty::Atur(pertama, pengaturan))
            }

            _ => None,
        };
    }

    fn esc_dispatch(&mut self, perantara: &[u8], abaikan: bool, aksi: u8) {
        if !abaikan && perantara.is_empty() && aksi == b'c' {
            self.perintah = Some(PerintahModeKitty::Reset);
        }
    }

    fn terminated(&self) -> bool {
        self.perintah.is_some()
    }
}

pub struct ParserModeKitty {
    parser: Parser,
}

impl ParserModeKitty {
    pub fn baru() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn terima(&mut self, data: &[u8]) -> (usize, Option<PerintahModeKitty>) {
        let mut penerima = PenerimaPerintah::default();

        let jumlah = self.parser.advance_until_terminated(&mut penerima, data);

        (jumlah, penerima.perintah)
    }
}
