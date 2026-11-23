/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Archive session
 * Purpose: Coordinates verified history archival for a terminal session
 */

use std::{thread, time::Duration};

use anyhow::{Result, bail, ensure};

use super::{HasilArsip, MotorTerminal, PengendaliArsip};

#[derive(Debug, PartialEq, Eq)]
pub enum HasilArsipSesi {
    Tersimpan(usize),
    Dihentikan(String),
}

pub struct PengarsipSesi {
    pengendali: PengendaliArsip,
    generasi_tertunda: Option<u64>,
    kesalahan: Option<String>,
}

impl PengarsipSesi {
    pub fn baru(pengendali: PengendaliArsip) -> Self {
        Self {
            pengendali,
            generasi_tertunda: None,
            kesalahan: None,
        }
    }

    pub fn kesalahan(&self) -> Option<&str> {
        self.kesalahan.as_deref()
    }

    pub fn ada_batch_tertunda(&self) -> bool {
        self.generasi_tertunda.is_some()
    }

    pub fn kirim_dari_motor(&mut self, motor: &MotorTerminal, jumlah: usize) -> Result<bool> {
        ensure!(
            self.kesalahan.is_none(),
            "Pengarsipan sesi telah dihentikan"
        );

        if self.ada_batch_tertunda() {
            return Ok(false);
        }

        let baris = motor.siapkan_batch_arsip(jumlah);

        if baris.is_empty() {
            return Ok(false);
        }

        let generasi = motor.generasi_riwayat();

        match self.pengendali.kirim_batch(baris) {
            Ok(true) => {
                self.generasi_tertunda = Some(generasi);

                Ok(true)
            }

            Ok(false) => Ok(false),

            Err(kesalahan) => {
                self.kesalahan = Some(format!("{kesalahan:#}"));

                Err(kesalahan)
            }
        }
    }

    pub fn periksa(&mut self, motor: &mut MotorTerminal) -> Option<HasilArsipSesi> {
        let hasil = self.pengendali.periksa()?;
        let generasi = self.generasi_tertunda.take();

        match hasil {
            HasilArsip::Tersimpan(baris) => {
                let sesuai = generasi.is_some_and(|generasi| {
                    motor.konfirmasi_batch_arsip_terverifikasi(generasi, &baris)
                });

                if sesuai {
                    Some(HasilArsipSesi::Tersimpan(baris.len()))
                } else {
                    Some(self.hentikan("Snapshot riwayat berubah setelah penulisan arsip"))
                }
            }

            HasilArsip::Gagal(pesan) => Some(self.hentikan(pesan)),
        }
    }

    fn hentikan(&mut self, pesan: impl Into<String>) -> HasilArsipSesi {
        let pesan = pesan.into();

        self.kesalahan = Some(pesan.clone());

        HasilArsipSesi::Dihentikan(pesan)
    }

    fn kirim_snapshot_akhir(&mut self, snapshot: Vec<String>) -> Result<()> {
        ensure!(
            self.kesalahan.is_none() && self.pengendali.kesalahan().is_none(),
            "Pengarsipan sebelumnya gagal; snapshot akhir tidak dikirim"
        );

        ensure!(
            !self.ada_batch_tertunda() && !self.pengendali.ada_batch_tertunda(),
            "Batch sebelumnya belum selesai"
        );

        ensure!(
            snapshot.iter().all(|teks| {
                teks.len() <= 16 * 1024
                    && !teks
                        .chars()
                        .any(|karakter| karakter.is_control() && karakter != '\t')
            }),
            "Snapshot akhir mengandung baris tidak valid"
        );

        let mut batch = Vec::new();
        let mut ukuran = 0usize;

        for teks in snapshot {
            if !batch.is_empty() && (batch.len() == 64 || ukuran + teks.len() > 64 * 1024) {
                self.kirim_batch_akhir(std::mem::take(&mut batch))?;
                ukuran = 0;
            }

            ukuran += teks.len();
            batch.push(teks);
        }

        if !batch.is_empty() {
            self.kirim_batch_akhir(batch)?;
        }

        Ok(())
    }
    fn kirim_batch_akhir(&mut self, baris: Vec<String>) -> Result<()> {
        ensure!(
            self.pengendali.kirim_batch(baris)?,
            "Batch akhir tidak dapat dikirim"
        );

        loop {
            match self.pengendali.periksa() {
                Some(HasilArsip::Tersimpan(_)) => return Ok(()),
                Some(HasilArsip::Gagal(pesan)) => {
                    bail!("Gagal menyimpan snapshot akhir: {pesan}")
                }
                None => thread::sleep(Duration::from_millis(1)),
            }
        }
    }

    pub fn selesaikan_dengan_snapshot(mut self, snapshot: Vec<String>) -> Result<()> {
        let hasil_snapshot = self.kirim_snapshot_akhir(snapshot);
        let hasil_penulis = self.pengendali.selesaikan();

        hasil_snapshot?;
        hasil_penulis
    }
    pub fn selesaikan(self) -> Result<()> {
        self.pengendali.selesaikan()
    }
}
