/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Pane layout
 * Purpose: Manages recursive pane layout, active selection, splitting and removal
 */

use super::manajer_sesi::IdSesi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArahPanel {
    Horizontal,
    Vertikal,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimpulPanel {
    Tunggal(IdSesi),

    Terbagi {
        arah: ArahPanel,
        pertama: Box<SimpulPanel>,
        kedua: Box<SimpulPanel>,
    },
}

impl SimpulPanel {
    fn memuat(&self, target: IdSesi) -> bool {
        match self {
            Self::Tunggal(id) => *id == target,

            Self::Terbagi { pertama, kedua, .. } => pertama.memuat(target) || kedua.memuat(target),
        }
    }

    fn kumpulkan(&self, daftar: &mut Vec<IdSesi>) {
        match self {
            Self::Tunggal(id) => daftar.push(*id),

            Self::Terbagi { pertama, kedua, .. } => {
                pertama.kumpulkan(daftar);
                kedua.kumpulkan(daftar);
            }
        }
    }

    fn pecah(&mut self, target: IdSesi, id_baru: IdSesi, arah: ArahPanel) -> bool {
        match self {
            Self::Tunggal(id) if *id == target => {
                *self = Self::Terbagi {
                    arah,
                    pertama: Box::new(Self::Tunggal(target)),
                    kedua: Box::new(Self::Tunggal(id_baru)),
                };

                true
            }

            Self::Tunggal(_) => false,

            Self::Terbagi { pertama, kedua, .. } => {
                pertama.pecah(target, id_baru, arah) || kedua.pecah(target, id_baru, arah)
            }
        }
    }

    fn hapus(self, target: IdSesi) -> Option<Self> {
        match self {
            Self::Tunggal(id) => {
                if id == target {
                    None
                } else {
                    Some(Self::Tunggal(id))
                }
            }

            Self::Terbagi {
                arah,
                pertama,
                kedua,
            } => match (pertama.hapus(target), kedua.hapus(target)) {
                (Some(pertama), Some(kedua)) => Some(Self::Terbagi {
                    arah,
                    pertama: Box::new(pertama),
                    kedua: Box::new(kedua),
                }),

                (Some(simpul), None) | (None, Some(simpul)) => Some(simpul),

                (None, None) => None,
            },
        }
    }
}

pub struct SusunanPanel {
    akar: SimpulPanel,
    aktif: IdSesi,
}

impl SusunanPanel {
    pub fn baru(id: IdSesi) -> Self {
        Self {
            akar: SimpulPanel::Tunggal(id),
            aktif: id,
        }
    }

    pub fn akar(&self) -> &SimpulPanel {
        &self.akar
    }

    pub fn id_aktif(&self) -> IdSesi {
        self.aktif
    }

    pub fn daftar_id(&self) -> Vec<IdSesi> {
        let mut daftar = Vec::new();

        self.akar.kumpulkan(&mut daftar);

        daftar
    }

    pub fn jumlah(&self) -> usize {
        self.daftar_id().len()
    }

    pub fn pilih(&mut self, id: IdSesi) -> bool {
        if !self.akar.memuat(id) {
            return false;
        }

        self.aktif = id;

        true
    }

    pub fn pecah(&mut self, id_baru: IdSesi, arah: ArahPanel) -> bool {
        if self.akar.memuat(id_baru) {
            return false;
        }

        if !self.akar.pecah(self.aktif, id_baru, arah) {
            return false;
        }

        self.aktif = id_baru;

        true
    }

    pub fn tutup(&mut self, id: IdSesi) -> bool {
        let daftar = self.daftar_id();

        if daftar.len() <= 1 {
            return false;
        }

        let Some(indeks) = daftar.iter().position(|item| *item == id) else {
            return false;
        };

        let pengganti = if self.aktif == id {
            if indeks + 1 < daftar.len() {
                daftar[indeks + 1]
            } else {
                daftar[indeks - 1]
            }
        } else {
            self.aktif
        };

        let akar = std::mem::replace(&mut self.akar, SimpulPanel::Tunggal(pengganti));

        self.akar = akar.hapus(id).expect("Susunan panel tidak boleh kosong");

        self.aktif = pengganti;

        true
    }
}
