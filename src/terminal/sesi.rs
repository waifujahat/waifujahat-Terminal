use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::{Context, Result, anyhow};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

const KUERI_KURSOR: &[u8] = b"\x1b[6n";
const RESPONS_KURSOR: &[u8] = b"\x1b[1;1R";

#[derive(Debug)]
pub enum PesanTerminal {
    Keluaran(Vec<u8>),
    Kesalahan(String),
    Berakhir,
}

pub struct SesiTerminal {
    _master: Box<dyn MasterPty + Send>,
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
                Self::jalankan_penulis(penulis, penerima_masukan, pengirim_kesalahan);
            })
            .context("Gagal membuat thread penulis")?;

        let pengirim_respons = pengirim_masukan.clone();

        thread::Builder::new()
            .name("waifujahat-pty-reader".into())
            .spawn(move || {
                Self::jalankan_pembaca(pembaca, pengirim_peristiwa, pengirim_respons);
            })
            .context("Gagal membuat thread pembaca")?;

        Ok(Self {
            _master: pasangan.master,
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
        self._master
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

    fn jalankan_penulis(
        mut penulis: Box<dyn Write + Send>,
        penerima: Receiver<Vec<u8>>,
        pengirim: Sender<PesanTerminal>,
    ) {
        while let Ok(masukan) = penerima.recv() {
            let hasil = penulis.write_all(&masukan).and_then(|_| penulis.flush());

            if let Err(kesalahan) = hasil {
                let _ = pengirim.send(PesanTerminal::Kesalahan(format!(
                    "Gagal menulis ke terminal: {kesalahan}"
                )));

                break;
            }
        }
    }

    fn jalankan_pembaca(
        mut pembaca: Box<dyn Read + Send>,
        pengirim: Sender<PesanTerminal>,
        pengirim_respons: Sender<Vec<u8>>,
    ) {
        let mut buffer = [0u8; 4096];
        let mut posisi_kueri = 0usize;

        loop {
            let jumlah = match pembaca.read(&mut buffer) {
                Ok(0) => break,
                Ok(jumlah) => jumlah,
                Err(kesalahan) => {
                    let _ = pengirim.send(PesanTerminal::Kesalahan(format!(
                        "Gagal membaca terminal: {kesalahan}"
                    )));

                    break;
                }
            };

            if std::env::var_os("WAIFUJAHAT_DEBUG_PTY").is_some() {
                eprintln!("[ConPTY RAW] {}", buffer[..jumlah].escape_ascii());
            }

            for &byte in &buffer[..jumlah] {
                if byte == KUERI_KURSOR[posisi_kueri] {
                    posisi_kueri += 1;

                    if posisi_kueri == KUERI_KURSOR.len() {
                        let _ = pengirim_respons.send(RESPONS_KURSOR.to_vec());
                        posisi_kueri = 0;
                    }
                } else {
                    posisi_kueri = usize::from(byte == KUERI_KURSOR[0]);
                }
            }

            if pengirim
                .send(PesanTerminal::Keluaran(buffer[..jumlah].to_vec()))
                .is_err()
            {
                break;
            }
        }

        let _ = pengirim.send(PesanTerminal::Berakhir);
    }
}

impl Drop for SesiTerminal {
    fn drop(&mut self) {
        let _ = self.proses.kill();
    }
}
