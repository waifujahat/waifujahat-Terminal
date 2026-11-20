/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Keyboard protocol
 * Purpose: Defines the active keyboard input protocol
 */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProtokolKeyboard {
    #[default]
    Xterm,
    Win32,
    Kitty,
}
