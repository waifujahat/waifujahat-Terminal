/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / SSH command builder
 * Purpose : Constructs validated OpenSSH command arguments
 */

use std::{ffi::OsString, path::Path};

use anyhow::{Context, Result, ensure};
use portable_pty::CommandBuilder;

use super::{ClientSsh, ProfilSsh};

const OPSI_SSH: &[&str] = &[
    "ForwardAgent=no",
    "ForwardX11=no",
    "ClearAllForwardings=yes",
    "PermitLocalCommand=no",
    "EnableEscapeCommandline=no",
    "BatchMode=no",
    "ConnectionAttempts=1",
    "ConnectTimeout=15",
    "ServerAliveInterval=30",
    "ServerAliveCountMax=3",
];

fn validasi_jalur_ssh(jalur: &Path) -> Result<&str> {
    ensure!(jalur.is_absolute(), "Jalur berkas SSH harus absolut");

    let teks = jalur.to_str().context("Jalur berkas SSH tidak valid")?;

    ensure!(
        !teks.contains('%')
            && !teks.contains("${")
            && !teks.contains('"')
            && !teks.chars().any(char::is_control),
        "Jalur berkas SSH mengandung karakter yang tidak didukung"
    );

    #[cfg(windows)]
    ensure!(
        matches!(
            jalur.components().next(),
            Some(std::path::Component::Prefix(prefix))
                if matches!(
                    prefix.kind(),
                    std::path::Prefix::Disk(_)
                )
        ),
        "Jalur berkas SSH harus menggunakan drive Windows"
    );

    ensure!(jalur.is_file(), "Berkas SSH tidak ditemukan");

    Ok(teks)
}

impl ProfilSsh {
    pub fn argumen(&self) -> Result<Vec<OsString>> {
        self.bangun_argumen(None)
    }

    fn bangun_argumen(&self, known_hosts: Option<&Path>) -> Result<Vec<OsString>> {
        self.validasi()?;

        let mut argumen = vec![
            OsString::from("-F"),
            OsString::from("none"),
            OsString::from("-tt"),
        ];

        if let Some(jalur) = known_hosts {
            let teks = validasi_jalur_ssh(jalur)?;

            let teks = if cfg!(windows) {
                teks.replace('\\', "/")
            } else {
                teks.to_owned()
            };

            argumen.extend([
                OsString::from("-o"),
                OsString::from("StrictHostKeyChecking=yes"),
                OsString::from("-o"),
                OsString::from(format!("UserKnownHostsFile=\"{teks}\"")),
                OsString::from("-o"),
                OsString::from("GlobalKnownHostsFile=none"),
                OsString::from("-o"),
                OsString::from("UpdateHostKeys=no"),
            ]);
        } else {
            argumen.extend([
                OsString::from("-o"),
                OsString::from("StrictHostKeyChecking=ask"),
            ]);
        }

        for opsi in OPSI_SSH {
            argumen.push(OsString::from("-o"));

            argumen.push(OsString::from(opsi));
        }

        if let Some(jalur) = &self.berkas_identitas {
            validasi_jalur_ssh(jalur)?;

            argumen.extend([
                OsString::from("-o"),
                OsString::from("IdentitiesOnly=yes"),
                OsString::from("-i"),
                jalur.as_os_str().to_owned(),
            ]);
        }

        argumen.extend([
            OsString::from("-l"),
            OsString::from(&self.pengguna),
            OsString::from("-p"),
            OsString::from(self.port.to_string()),
            OsString::from("--"),
            OsString::from(&self.host),
        ]);

        Ok(argumen)
    }
}

impl ClientSsh {
    pub fn perintah(&self, profil: &ProfilSsh) -> Result<CommandBuilder> {
        self.buat_perintah(profil, None)
    }

    pub fn perintah_dengan_known_hosts(
        &self,
        profil: &ProfilSsh,
        known_hosts: &Path,
    ) -> Result<CommandBuilder> {
        self.buat_perintah(profil, Some(known_hosts))
    }

    fn buat_perintah(
        &self,
        profil: &ProfilSsh,
        known_hosts: Option<&Path>,
    ) -> Result<CommandBuilder> {
        let argumen = profil.bangun_argumen(known_hosts)?;

        let mut perintah = CommandBuilder::new(self.program());

        perintah.args(argumen);

        perintah.env("SSH_ASKPASS_REQUIRE", "never");

        Ok(perintah)
    }
}
