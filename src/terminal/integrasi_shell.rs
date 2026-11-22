/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Shell integration
 * Purpose : Parses OSC 133 lifecycle and OSC 7 working directory sequences
 */

use std::path::{Path, PathBuf};

use vte::{Parser, Perform};

mod direktori;

use direktori::uraikan_direktori;

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
    direktori: Option<PathBuf>,
}

impl PenerimaPenanda {
    fn tangani_direktori(&mut self, parameter: &[&[u8]]) {
        if parameter.len() != 2 {
            return;
        }

        if let Some(direktori) = uraikan_direktori(parameter[1]) {
            self.direktori = Some(direktori);
        }
    }

    fn tangani_penanda_shell(&mut self, parameter: &[&[u8]]) {
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

impl Perform for PenerimaPenanda {
    fn osc_dispatch(&mut self, parameter: &[&[u8]], _diakhiri_bel: bool) {
        match parameter.first().copied() {
            Some(b"7") => {
                self.tangani_direktori(parameter);
            }

            Some(b"133") => {
                self.tangani_penanda_shell(parameter);
            }

            _ => {}
        }
    }
}

pub struct ParserIntegrasiShell {
    parser: Parser,
    direktori_terakhir: Option<PathBuf>,
}

impl ParserIntegrasiShell {
    pub fn baru() -> Self {
        Self {
            parser: Parser::new(),
            direktori_terakhir: None,
        }
    }

    pub fn terima(&mut self, data: &[u8]) -> Vec<PenandaShell> {
        let mut penerima = PenerimaPenanda::default();

        self.parser.advance(&mut penerima, data);

        if let Some(direktori) = penerima.direktori {
            self.direktori_terakhir = Some(direktori);
        }

        penerima.penanda
    }

    pub fn direktori_terakhir(&self) -> Option<&Path> {
        self.direktori_terakhir.as_deref()
    }
}
