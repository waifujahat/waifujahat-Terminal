/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Build / Windows resources
 * Purpose: Embeds the application icon
 */

use std::{env, io};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets/icons/ICONTERMINAL.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut sumber_daya = winresource::WindowsResource::new();

        sumber_daya
            .set_icon("assets/icons/ICONTERMINAL.ico")
            .set("FileDescription", "Waifujahat Terminal")
            .set("ProductName", "Waifujahat Terminal")
            .compile()?;
    }

    Ok(())
}
