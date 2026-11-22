/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / SSH profile
 * Purpose : Defines and validates SSH connection profiles
 */

use std::{net::IpAddr, path::PathBuf};

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilSsh {
    pub id: String,

    #[serde(rename = "name")]
    pub nama: String,

    pub host: String,

    #[serde(rename = "username")]
    pub pengguna: String,

    #[serde(default = "port_default")]
    pub port: u16,

    #[serde(
        rename = "identity_file",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub berkas_identitas: Option<PathBuf>,
}

fn port_default() -> u16 {
    22
}

impl ProfilSsh {
    pub fn validasi(&self) -> Result<()> {
        ensure!(Self::id_valid(&self.id), "Identitas profil SSH tidak valid");

        ensure!(
            !self.nama.is_empty()
                && self.nama == self.nama.trim()
                && self.nama.chars().count() <= 64
                && !self.nama.chars().any(char::is_control),
            "Nama profil SSH tidak valid"
        );

        ensure!(Self::host_valid(&self.host), "Host SSH tidak valid");

        ensure!(
            Self::pengguna_valid(&self.pengguna),
            "Username SSH tidak valid"
        );

        ensure!(self.port != 0, "Port SSH tidak valid");

        if let Some(jalur) = &self.berkas_identitas {
            ensure!(
                jalur.is_absolute(),
                "Jalur berkas identitas SSH harus absolut"
            );

            ensure!(
                jalur
                    .to_str()
                    .is_some_and(|teks| { !teks.chars().any(char::is_control) }),
                "Jalur berkas identitas SSH tidak valid"
            );
        }

        Ok(())
    }

    pub(crate) fn id_valid(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= 64
            && id
                .bytes()
                .next()
                .is_some_and(|karakter| karakter.is_ascii_lowercase() || karakter.is_ascii_digit())
            && id.bytes().all(|karakter| {
                karakter.is_ascii_lowercase()
                    || karakter.is_ascii_digit()
                    || matches!(karakter, b'-' | b'_')
            })
    }

    fn host_valid(host: &str) -> bool {
        if host.parse::<IpAddr>().is_ok() {
            return true;
        }

        let host = host.strip_suffix('.').unwrap_or(host);

        if host.is_empty() || host.len() > 253 {
            return false;
        }

        if host
            .bytes()
            .all(|karakter| karakter.is_ascii_digit() || karakter == b'.')
        {
            return false;
        }

        host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|karakter| karakter.is_ascii_alphanumeric() || karakter == b'-')
        })
    }

    fn pengguna_valid(pengguna: &str) -> bool {
        !pengguna.is_empty()
            && pengguna.len() <= 64
            && pengguna
                .bytes()
                .next()
                .is_some_and(|karakter| karakter.is_ascii_alphanumeric() || karakter == b'_')
            && pengguna.bytes().all(|karakter| {
                karakter.is_ascii_alphanumeric() || matches!(karakter, b'_' | b'-' | b'.')
            })
    }
}
