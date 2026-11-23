/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Pending PTY messages
 * Purpose: Preserves event order while terminal output is deferred
 */

use std::collections::VecDeque;

use super::PesanTerminal;

#[derive(Default)]
pub struct AntreanPesanTerminal {
    tertunda: VecDeque<PesanTerminal>,
}

impl AntreanPesanTerminal {
    pub fn ada_tertunda(&self) -> bool {
        !self.tertunda.is_empty()
    }

    pub fn ambil(
        &mut self,
        sumber: impl FnOnce() -> Option<PesanTerminal>,
    ) -> Option<PesanTerminal> {
        self.tertunda.pop_front().or_else(sumber)
    }

    pub fn tunda(&mut self, pesan: PesanTerminal) {
        self.tertunda.push_back(pesan);
    }

    pub fn tunda_di_depan(&mut self, pesan: PesanTerminal) {
        self.tertunda.push_front(pesan);
    }
}
