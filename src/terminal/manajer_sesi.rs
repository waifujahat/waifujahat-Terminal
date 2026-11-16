/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Session manager
 * Purpose: Manages session identities, ownership and active selection
 */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IdSesi(u64);

impl IdSesi {
    pub fn nilai(self) -> u64 {
        self.0
    }
}

struct EntriSesi<T> {
    id: IdSesi,
    isi: T,
}

pub struct ManajerSesi<T> {
    daftar: Vec<EntriSesi<T>>,
    id_aktif: Option<IdSesi>,
    id_berikutnya: u64,
}

impl<T> ManajerSesi<T> {
    pub fn baru() -> Self {
        Self {
            daftar: Vec::new(),
            id_aktif: None,
            id_berikutnya: 1,
        }
    }

    pub fn geser_aktif(&mut self, mundur: bool) -> bool {
        if self.daftar.len() < 2 {
            return false;
        }

        let Some(id) = self.id_aktif else {
            return false;
        };

        let Some(indeks) = self.daftar.iter().position(|entri| entri.id == id) else {
            return false;
        };

        let jumlah = self.daftar.len();

        let tujuan = if mundur {
            if indeks == 0 { jumlah - 1 } else { indeks - 1 }
        } else {
            (indeks + 1) % jumlah
        };
        self.id_aktif = Some(self.daftar[tujuan].id);

        true
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (IdSesi, &mut T)> + '_ {
        self.daftar
            .iter_mut()
            .map(|entri| (entri.id, &mut entri.isi))
    }

    pub fn jumlah(&self) -> usize {
        self.daftar.len()
    }

    pub fn kosong(&self) -> bool {
        self.daftar.is_empty()
    }

    pub fn id_aktif(&self) -> Option<IdSesi> {
        self.id_aktif
    }

    pub fn daftar_id(&self) -> impl Iterator<Item = IdSesi> + '_ {
        self.daftar.iter().map(|entri| entri.id)
    }

    pub fn tambah(&mut self, isi: T) -> IdSesi {
        let id = IdSesi(self.id_berikutnya);

        self.id_berikutnya = self
            .id_berikutnya
            .checked_add(1)
            .expect("Identitas sesi telah mencapai batas");

        self.daftar.push(EntriSesi { id, isi });
        self.id_aktif = Some(id);

        id
    }

    pub fn ambil(&self, id: IdSesi) -> Option<&T> {
        self.daftar
            .iter()
            .find(|entri| entri.id == id)
            .map(|entri| &entri.isi)
    }

    pub fn ambil_mut(&mut self, id: IdSesi) -> Option<&mut T> {
        self.daftar
            .iter_mut()
            .find(|entri| entri.id == id)
            .map(|entri| &mut entri.isi)
    }

    pub fn aktif(&self) -> Option<&T> {
        self.ambil(self.id_aktif?)
    }

    pub fn aktif_mut(&mut self) -> Option<&mut T> {
        self.ambil_mut(self.id_aktif?)
    }

    pub fn pilih(&mut self, id: IdSesi) -> bool {
        if self.ambil(id).is_none() {
            return false;
        }

        self.id_aktif = Some(id);

        true
    }

    pub fn tutup(&mut self, id: IdSesi) -> Option<T> {
        let indeks = self.daftar.iter().position(|entri| entri.id == id)?;

        let entri = self.daftar.remove(indeks);

        if self.id_aktif == Some(id) {
            self.id_aktif = self
                .daftar
                .get(indeks)
                .or_else(|| self.daftar.last())
                .map(|entri| entri.id);
        }

        Some(entri.isi)
    }
}
