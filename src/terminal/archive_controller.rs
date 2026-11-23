/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Archive controller
 * Purpose: Coordinates archive batches and asynchronous acknowledgments
 */

use std::sync::mpsc::{Receiver, TryRecvError};

use anyhow::{Result, ensure};

use super::PenulisArsip;

#[derive(Debug, PartialEq, Eq)]
pub enum HasilArsip {
    Tersimpan(Vec<String>),
    Gagal(String),
}

struct BatchTertunda {
    baris: Vec<String>,
    konfirmasi: Receiver<std::result::Result<(), String>>,
}

pub struct PengendaliArsip {
    penulis: PenulisArsip,
    tertunda: Option<BatchTertunda>,
    kesalahan: Option<String>,
}

impl PengendaliArsip {
    pub fn baru(penulis: PenulisArsip) -> Self {
        Self {
            penulis,
            tertunda: None,
            kesalahan: None,
        }
    }

    pub fn ada_batch_tertunda(&self) -> bool {
        self.tertunda.is_some()
    }

    pub fn kesalahan(&self) -> Option<&str> {
        self.kesalahan.as_deref()
    }

    pub fn kirim_batch(&mut self, baris: Vec<String>) -> Result<bool> {
        ensure!(
            self.kesalahan.is_none(),
            "Pengendali arsip tidak dapat digunakan setelah gagal"
        );

        if self.tertunda.is_some() {
            return Ok(false);
        }

        let konfirmasi = self.penulis.kirim_batch_dengan_konfirmasi(baris.clone())?;

        self.tertunda = Some(BatchTertunda { baris, konfirmasi });

        Ok(true)
    }

    pub fn periksa(&mut self) -> Option<HasilArsip> {
        let hasil = match self.tertunda.as_ref()?.konfirmasi.try_recv() {
            Ok(Ok(())) => Ok(()),

            Ok(Err(pesan)) => Err(pesan),

            Err(TryRecvError::Empty) => return None,

            Err(TryRecvError::Disconnected) => Err("Kanal konfirmasi arsip terputus".to_owned()),
        };

        let tertunda = self.tertunda.take()?;

        match hasil {
            Ok(()) => Some(HasilArsip::Tersimpan(tertunda.baris)),

            Err(pesan) => {
                self.kesalahan = Some(pesan.clone());

                Some(HasilArsip::Gagal(pesan))
            }
        }
    }

    pub fn selesaikan(self) -> Result<()> {
        self.penulis.selesaikan()
    }
}
