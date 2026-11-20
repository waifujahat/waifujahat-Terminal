/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Assets
 * Purpose: Provides embedded application assets
 */

use std::{borrow::Cow, sync::Arc};

use gpui::{App, AssetSource, Global, Image, ImageFormat, RenderImage, Result, SharedString};

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

    pub(crate) fn gambar(&self) -> Arc<RenderImage> {
        Arc::clone(&self.gambar)
    }
}

impl AssetSource for AsetAplikasi {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let data: Option<&'static [u8]> = match path {
            "icons/search.svg" => Some(include_bytes!("../assets/icons/search.svg")),
            "images/demo.jpg" => Some(GAMBAR_LATAR),

            _ => None,
        };

        Ok(data.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let daftar = ["icons/search.svg", "images/demo.jpg"]
            .into_iter()
            .filter(|nama| nama.starts_with(path))
            .map(SharedString::from)
            .collect();

        Ok(daftar)
    }
}
