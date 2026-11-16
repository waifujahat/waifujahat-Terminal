/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Shell integration
 * Purpose: Parses OSC 133 shell integration sequences
 */

use vte::{Parser, Perform};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PenandaShell {
    AwalPrompt,
    AwalInput,
    AwalKeluaran,
    PerintahSelesai(Option<i32>),
}

#[derive(Default)]
struct PenerimaPenanda {
    penanda: Vec<PenandaShell>,
}

impl Perform for PenerimaPenanda {
    fn osc_dispatch(&mut self, parameter: &[&[u8]], _diakhiri_bel: bool) {
        if parameter.first().copied() != Some(b"133".as_slice()) {
            return;
        }

        let Some(kode) = parameter.get(1) else {
            return;
        };

        let penanda = match *kode {
            b"A" => PenandaShell::AwalPrompt,
            b"B" => PenandaShell::AwalInput,
            b"C" => PenandaShell::AwalKeluaran,

            b"D" => {
                let kode_keluar = match parameter.get(2) {
                    Some(nilai) => {
                        let Some(kode) = std::str::from_utf8(nilai)
                            .ok()
                            .and_then(|teks| teks.parse::<i32>().ok())
                        else {
                            return;
                        };

                        Some(kode)
                    }

                    None => None,
                };

                PenandaShell::PerintahSelesai(kode_keluar)
            }

            _ => return,
        };

        self.penanda.push(penanda);
    }
}

pub struct ParserIntegrasiShell {
    parser: Parser,
}

impl ParserIntegrasiShell {
    pub fn baru() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn terima(&mut self, data: &[u8]) -> Vec<PenandaShell> {
        let mut penerima = PenerimaPenanda::default();

        self.parser.advance(&mut penerima, data);

        penerima.penanda
    }
}
