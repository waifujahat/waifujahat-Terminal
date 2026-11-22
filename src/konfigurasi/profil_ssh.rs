/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Configuration / SSH profiles
 * Purpose : Manages persistent SSH connection profiles
 */

use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::terminal::ProfilSsh;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaftarProfilSsh {
    #[serde(rename = "version")]
    pub versi: u32,

    #[serde(rename = "profiles", default)]
    pub profil: Vec<ProfilSsh>,
}

impl Default for DaftarProfilSsh {
    fn default() -> Self {
        Self {
            versi: Self::VERSI,
            profil: Vec::new(),
        }
    }
}

impl DaftarProfilSsh {
    pub const VERSI: u32 = 1;
    pub const BATAS_PROFIL: usize = 128;

    pub fn validasi(&self) -> Result<()> {
        ensure!(
            self.versi == Self::VERSI,
            "Versi konfigurasi SSH tidak didukung"
        );

        ensure!(
            self.profil.len() <= Self::BATAS_PROFIL,
            "Jumlah profil SSH melebihi batas"
        );

        let mut identitas = HashSet::new();

        for (indeks, profil) in self.profil.iter().enumerate() {
            profil
                .validasi()
                .with_context(|| format!("Profil SSH ke-{} tidak valid", indeks + 1))?;

            ensure!(
                identitas.insert(profil.id.as_str()),
                "Identitas profil SSH duplikat : {}",
                profil.id
            );
        }

        Ok(())
    }

    pub fn cari(&self, id: &str) -> Option<&ProfilSsh> {
        self.profil.iter().find(|profil| profil.id == id)
    }
}
