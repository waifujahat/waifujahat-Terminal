/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Asynchronous archive writer
 * Purpose: Writes session history using a bounded background queue
 */

use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError},
    thread::{self, JoinHandle},
};

use anyhow::{Context, Result, anyhow, ensure};

use super::ArsipRiwayat;

const KAPASITAS_ANTREAN: usize = 256;
const BATAS_PANJANG_BARIS: usize = 16 * 1024;
const BATAS_BARIS_BATCH: usize = 64;
const BATAS_UKURAN_BATCH: usize = 64 * 1024;

struct PerintahArsip {
    baris: Vec<String>,
    konfirmasi: Sender<std::result::Result<(), String>>,
}

pub struct PenulisArsip {
    arsip_penghapusan: ArsipRiwayat,
    pengirim: Option<SyncSender<PerintahArsip>>,
    penerima_kesalahan: Receiver<String>,
    pekerja: Option<JoinHandle<Result<()>>>,
}

pub struct TugasPenghapusanArsip {
    pekerja: Option<JoinHandle<Result<()>>>,
}

impl TugasPenghapusanArsip {
    pub fn periksa(&mut self) -> Option<Result<()>> {
        if !self.pekerja.as_ref()?.is_finished() {
            return None;
        }

        let pekerja = self.pekerja.take()?;

        Some(
            pekerja
                .join()
                .map_err(|_| anyhow!("Tugas penghapusan arsip mengalami panic"))
                .and_then(|hasil| hasil),
        )
    }

    pub fn tunggu(mut self) -> Result<()> {
        let pekerja = self
            .pekerja
            .take()
            .context("Tugas penghapusan arsip tidak tersedia")?;

        pekerja
            .join()
            .map_err(|_| anyhow!("Tugas penghapusan arsip mengalami panic"))?
    }
}

impl PenulisArsip {
    pub fn baru(arsip: ArsipRiwayat) -> Result<Self> {
        let arsip_penghapusan = arsip.clone();
        let (pengirim, penerima) = mpsc::sync_channel::<PerintahArsip>(KAPASITAS_ANTREAN);

        let (pengirim_kesalahan, penerima_kesalahan) = mpsc::channel::<String>();

        let pekerja = thread::Builder::new()
            .name("waifujahat-arsip".to_owned())
            .spawn(move || {
                for perintah in penerima {
                    // Validasi seluruh batch sebelum menulis.
                    let hasil = arsip
                        .validasi_batch(&perintah.baris)
                        .and_then(|()| {
                            perintah
                                .baris
                                .iter()
                                .try_for_each(|baris| arsip.simpan_baris(baris))
                        })
                        .and_then(|()| arsip.sinkronkan());

                    match hasil {
                        Ok(()) => {
                            let _ = perintah.konfirmasi.send(Ok(()));
                        }

                        Err(kesalahan) => {
                            let pesan = format!("{kesalahan:#}");

                            let _ = pengirim_kesalahan.send(pesan.clone());

                            let _ = perintah.konfirmasi.send(Err(pesan));

                            return Err(kesalahan);
                        }
                    }
                }

                Ok(())
            })
            .context("Gagal membuat pekerja arsip riwayat")?;

        Ok(Self {
            arsip_penghapusan,
            pengirim: Some(pengirim),
            penerima_kesalahan,
            pekerja: Some(pekerja),
        })
    }

    pub fn kirim_baris(&self, teks: &str) -> Result<()> {
        self.kirim_batch(vec![teks.to_owned()])
    }

    pub fn kirim_batch(&self, baris: Vec<String>) -> Result<()> {
        self.kirim_batch_dengan_konfirmasi(baris).map(|_| ())
    }

    pub fn kirim_batch_dengan_konfirmasi(
        &self,
        baris: Vec<String>,
    ) -> Result<Receiver<std::result::Result<(), String>>> {
        ensure!(!baris.is_empty(), "Kumpulan baris arsip kosong");

        ensure!(
            baris.len() <= BATAS_BARIS_BATCH,
            "Jumlah baris arsip melebihi batas batch"
        );

        let mut ukuran_total = 0usize;

        for teks in &baris {
            ensure!(
                teks.len() <= BATAS_PANJANG_BARIS,
                "Baris arsip melebihi batas antrean"
            );

            ukuran_total = ukuran_total
                .checked_add(teks.len())
                .context("Ukuran kumpulan baris arsip tidak valid")?;
        }

        ensure!(
            ukuran_total <= BATAS_UKURAN_BATCH,
            "Ukuran kumpulan baris arsip melebihi batas"
        );

        let pengirim = self
            .pengirim
            .as_ref()
            .context("Pekerja arsip sudah ditutup")?;

        let (pengirim_konfirmasi, penerima_konfirmasi) = mpsc::channel();

        let perintah = PerintahArsip {
            baris,
            konfirmasi: pengirim_konfirmasi,
        };

        pengirim
            .try_send(perintah)
            .map_err(|kesalahan| match kesalahan {
                TrySendError::Full(_) => {
                    anyhow!("Antrean arsip penuh; batch tidak diterima")
                }

                TrySendError::Disconnected(_) => {
                    anyhow!("Pekerja arsip terputus; batch tidak diterima")
                }
            })?;

        Ok(penerima_konfirmasi)
    }

    pub fn ambil_kesalahan(&self) -> Option<String> {
        self.penerima_kesalahan.try_recv().ok()
    }

    fn hentikan_pekerja(&mut self) -> Result<()> {
        self.pengirim.take();

        let pekerja = self
            .pekerja
            .take()
            .context("Pekerja arsip tidak tersedia")?;

        pekerja
            .join()
            .map_err(|_| anyhow!("Pekerja arsip berhenti karena panic"))?
    }

    pub fn selesaikan(mut self) -> Result<()> {
        self.hentikan_pekerja()
    }

    pub fn mulai_penghapusan_arsip(self) -> Result<TugasPenghapusanArsip> {
        let pekerja = thread::Builder::new()
            .name("waifujahat-penghapusan-arsip".to_owned())
            .spawn(move || self.selesaikan_dan_hapus_arsip())
            .context("Gagal memulai tugas penghapusan arsip")?;

        Ok(TugasPenghapusanArsip {
            pekerja: Some(pekerja),
        })
    }

    pub fn selesaikan_dan_hapus_arsip(mut self) -> Result<()> {
        let hasil_pekerja = self.hentikan_pekerja();
        let hasil_penghapusan = self.arsip_penghapusan.hapus_semua_berkas();

        match (hasil_pekerja, hasil_penghapusan) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(kesalahan), Ok(())) => Err(kesalahan),
            (Ok(()), Err(kesalahan)) => Err(kesalahan),
            (Err(pekerja), Err(hapus)) => Err(anyhow!(
                "Pekerja arsip gagal: {pekerja:#}; penghapusan arsip gagal: {hapus:#}"
            )),
        }
    }
}
