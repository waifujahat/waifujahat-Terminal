/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Bracketed paste
 * Purpose: Encodes clipboard text for terminal applications
 */

pub fn sandi_tempel(teks: &str, berbingkai: bool) -> Vec<u8> {
    let teks = teks.replace("\r\n", "\n").replace('\r', "\n");

    if !berbingkai {
        return teks.replace('\n', " ").into_bytes();
    }

    let teks = teks.replace('\x1b', "");

    format!("\x1b[200~{teks}\x1b[201~").into_bytes()
}
