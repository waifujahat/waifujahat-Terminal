/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Application / Assets
 * Purpose: Provides embedded application assets
 */

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub(crate) struct AsetAplikasi;

impl AssetSource for AsetAplikasi {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let data: Option<&'static [u8]> = match path {
            "icons/search.svg" => Some(include_bytes!("../assets/icons/search.svg")),

            _ => None,
        };

        Ok(data.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let daftar = ["icons/search.svg"]
            .into_iter()
            .filter(|nama| nama.starts_with(path))
            .map(SharedString::from)
            .collect();

        Ok(daftar)
    }
}
