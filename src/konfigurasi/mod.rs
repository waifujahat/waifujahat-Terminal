/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration
 * Purpose : Exposes persistent terminal configuration
 */

mod geometri;
mod pengaturan;
mod penyimpanan;
mod validasi;

pub use pengaturan::{PengaturanTerminal, PilihanShell};
pub use penyimpanan::PenyimpananPengaturan;
