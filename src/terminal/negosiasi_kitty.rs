/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Kitty negotiation
 * Purpose: Coordinates keyboard protocol negotiation and screen mode isolation
 */

use super::{ModeKeyboardKitty, ParserModeKitty, PerintahModeKitty};

pub(super) struct NegosiasiKitty {
    parser: ParserModeKitty,
    utama: ModeKeyboardKitty,
    alternatif: ModeKeyboardKitty,
}

impl NegosiasiKitty {
    pub(super) fn baru() -> Self {
        Self {
            parser: ParserModeKitty::baru(),
            utama: ModeKeyboardKitty::baru(),
            alternatif: ModeKeyboardKitty::baru(),
        }
    }

    pub(super) fn terima(&mut self, data: &[u8]) -> (usize, Option<PerintahModeKitty>) {
        self.parser.terima(data)
    }

    pub(super) fn mode(&self, alternatif: bool) -> &ModeKeyboardKitty {
        if alternatif {
            &self.alternatif
        } else {
            &self.utama
        }
    }

    pub(super) fn proses(
        &mut self,
        perintah: PerintahModeKitty,
        alternatif: bool,
    ) -> Option<Vec<u8>> {
        let mode = if alternatif {
            &mut self.alternatif
        } else {
            &mut self.utama
        };

        match perintah {
            PerintahModeKitty::Tanya => {
                return Some(mode.respons());
            }

            PerintahModeKitty::Dorong(flag) => {
                mode.dorong(flag);
            }

            PerintahModeKitty::Keluarkan(jumlah) => {
                mode.keluarkan(jumlah);
            }

            PerintahModeKitty::Atur(flag, pengaturan) => {
                mode.atur(flag, pengaturan);
            }

            PerintahModeKitty::Reset => {
                self.utama.reset();
                self.alternatif.reset();
            }

            PerintahModeKitty::PeralihanLayar => {}
        }

        None
    }
}
