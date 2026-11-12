/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PTY session
 * Purpose: Owns the PowerShell process, PTY, resizing, and worker lifetime.
 */

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

mod io;

use io::{jalankan_pembaca, jalankan_penulis};

use anyhow::{Context, Result, anyhow};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

#[derive(Debug)]
pub enum PesanTerminal {
    Keluaran(Vec<u8>),
    Kesalahan(String),
    Berakhir,
}

pub struct SesiTerminal {
    master: Box<dyn MasterPty + Send>,
    proses: Box<dyn Child + Send + Sync>,
    pengirim_masukan: Sender<Vec<u8>>,
    penerima_peristiwa: Receiver<PesanTerminal>,
}

impl SesiTerminal {
    pub fn baru() -> Result<Self> {
        let sistem_pty = native_pty_system();

        let pasangan = sistem_pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal membuat pseudoterminal")?;

        let mut perintah = CommandBuilder::new("powershell.exe");

        perintah.arg("-NoLogo");
        perintah.arg("-NoProfile");

        let proses = pasangan
            .slave
            .spawn_command(perintah)
            .context("Gagal menjalankan PowerShell")?;

        drop(pasangan.slave);

        let pembaca = pasangan
            .master
            .try_clone_reader()
            .context("Gagal membuka pembaca terminal")?;

        let penulis = pasangan
            .master
            .take_writer()
            .context("Gagal membuka penulis terminal")?;

        let (pengirim_masukan, penerima_masukan) = mpsc::channel();
        let (pengirim_peristiwa, penerima_peristiwa) = mpsc::channel();

        let pengirim_kesalahan = pengirim_peristiwa.clone();

        thread::Builder::new()
            .name("waifujahat-pty-writer".into())
            .spawn(move || {
                jalankan_penulis(penulis, penerima_masukan, pengirim_kesalahan);
            })
            .context("Gagal membuat thread penulis")?;

        let pengirim_respons = pengirim_masukan.clone();

        thread::Builder::new()
            .name("waifujahat-pty-reader".into())
            .spawn(move || {
                jalankan_pembaca(pembaca, pengirim_peristiwa, pengirim_respons);
            })
            .context("Gagal membuat thread pembaca")?;

        Ok(Self {
            master: pasangan.master,
            proses,
            pengirim_masukan,
            penerima_peristiwa,
        })
    }

    pub fn kirim(&self, masukan: &str) -> Result<()> {
        self.kirim_bytes(masukan.as_bytes())
    }

    pub fn kirim_bytes(&self, masukan: &[u8]) -> Result<()> {
        self.pengirim_masukan
            .send(masukan.to_vec())
            .map_err(|_| anyhow!("Koneksi input terminal terputus"))
    }

    pub fn ubah_ukuran(&self, kolom: u16, baris: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows: baris.max(1),
                cols: kolom.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal mengubah ukuran ConPTY")
    }

    pub fn try_ambil(&self) -> Option<PesanTerminal> {
        self.penerima_peristiwa.try_recv().ok()
    }


}

impl Drop for SesiTerminal {
    fn drop(&mut self) {
        let _ = self.proses.kill();
    }
}
