/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PTY I/O workers
 * Purpose: Manages PTY reader and writer loops.
 */

use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};

use super::PesanTerminal;

const KUERI_KURSOR: &[u8] = b"\x1b[6n";
const RESPONS_KURSOR: &[u8] = b"\x1b[1;1R";

pub(super) fn jalankan_penulis(
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

pub(super) fn jalankan_pembaca(
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
