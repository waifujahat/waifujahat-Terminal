/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Assets
 * Purpose : Provides embedded and configurable application assets
 */

use std::{borrow::Cow, fs, path::Path, sync::Arc};

use anyhow::{Context, bail};

use gpui::{App, AssetSource, Global, Image, ImageFormat, RenderImage, Result, SharedString};

use waifujahat_terminal::konfigurasi::PengaturanTerminal;

const GAMBAR_LATAR: &[u8] = include_bytes!("../assets/images/demo.jpg");

pub(crate) struct AsetAplikasi;

pub(crate) struct LatarAplikasi {
    gambar: Arc<RenderImage>,
}

impl Global for LatarAplikasi {}

impl LatarAplikasi {
    pub(crate) fn muat(aplikasi: &mut App) -> Result<()> {
        let gambar = Image::from_bytes(ImageFormat::Jpeg, GAMBAR_LATAR.to_vec())
            .to_image_data(aplikasi.svg_renderer())?;

        aplikasi.set_global(Self { gambar });

        Ok(())
    }

    pub(crate) fn muat_dengan_pengaturan(
        aplikasi: &mut App,
        pengaturan: &PengaturanTerminal,
    ) -> Result<()> {
        if let Some(jalur) = &pengaturan.gambar_latar {
            match Self::muat_gambar(aplikasi, jalur) {
                Ok(gambar) => {
                    aplikasi.set_global(Self { gambar });

                    return Ok(());
                }

                Err(kesalahan) => {
                    eprintln!("Gagal memuat wallpaper : {Error:#}", Error = kesalahan);

                    eprintln!("Menggunakan wallpaper default");
                }
            }
        }

        Self::muat(aplikasi)
    }

    fn muat_gambar(aplikasi: &App, jalur: &Path) -> Result<Arc<RenderImage>> {
        let ekstensi = jalur
            .extension()
            .and_then(|ekstensi| ekstensi.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let format = match ekstensi.as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpeg,

            "png" => ImageFormat::Png,

            _ => bail!("Format wallpaper tidak didukung"),
        };

        let data = fs::read(jalur)
            .with_context(|| format!("Gagal membaca wallpaper : {}", jalur.display()))?;

        Image::from_bytes(format, data)
            .to_image_data(aplikasi.svg_renderer())
            .with_context(|| format!("Gagal melakukan decoding wallpaper : {}", jalur.display()))
    }

    pub(crate) fn gambar(&self) -> Arc<RenderImage> {
        Arc::clone(&self.gambar)
    }
}

impl AssetSource for AsetAplikasi {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let data: Option<&'static [u8]> = match path {
            "icons/search.svg" => Some(include_bytes!("../assets/icons/search.svg")),

            "icons/edit.svg" => Some(include_bytes!("../assets/icons/edit.svg")),

            "images/demo.jpg" => Some(GAMBAR_LATAR),

            _ => None,
        };

        Ok(data.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let daftar = ["icons/search.svg", "icons/edit.svg", "images/demo.jpg"]
            .into_iter()
            .filter(|nama| nama.starts_with(path))
            .map(SharedString::from)
            .collect();

        Ok(daftar)
    }
}
