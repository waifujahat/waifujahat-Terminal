/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Output buffer
 * Purpose: Processes bounded PTY output incrementally
 */

use anyhow::{Result, ensure};

const BATAS_BUFFER: usize = 4096;

#[derive(Debug)]
pub struct BufferKeluaran {
    data: Vec<u8>,
    posisi: usize,
}

impl BufferKeluaran {
    pub fn baru(data: Vec<u8>) -> Result<Self> {
        ensure!(!data.is_empty(), "Buffer keluaran tidak boleh kosong");

        ensure!(data.len() <= BATAS_BUFFER, "Buffer keluaran melebihi batas");

        Ok(Self { data, posisi: 0 })
    }

    pub fn sisa_byte(&self) -> usize {
        self.data.len().saturating_sub(self.posisi)
    }

    pub fn selesai(&self) -> bool {
        self.sisa_byte() == 0
    }

    pub fn ambil_potongan(&mut self, batas: usize) -> Result<Option<Vec<u8>>> {
        ensure!(
            (1..=BATAS_BUFFER).contains(&batas),
            "Batas potongan keluaran tidak valid"
        );

        if self.selesai() {
            return Ok(None);
        }

        let akhir_batas = self.posisi.saturating_add(batas).min(self.data.len());

        let akhir = self.data[self.posisi..akhir_batas]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(akhir_batas, |indeks| self.posisi + indeks + 1);

        let potongan = self.data[self.posisi..akhir].to_vec();

        self.posisi = akhir;

        Ok(Some(potongan))
    }
}
