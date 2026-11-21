/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Shell process monitor
 * Purpose : Monitors shell processes outside the UI thread
 */

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TrySendError};
use std::thread;
use std::time::Duration;

use anyhow::Result;

use super::proses_shell::{ProsesTerminal, potret_proses};

const INTERVAL_PEMANTAUAN: Duration = Duration::from_millis(500);

pub struct PemantauProsesShell {
    penerima: Receiver<Result<Vec<ProsesTerminal>>>,
    _pengirim_henti: Sender<()>,
}

impl PemantauProsesShell {
    pub fn baru() -> Result<Self> {
        let (pengirim, penerima) = mpsc::sync_channel(1);

        let (pengirim_henti, penerima_henti) = mpsc::channel();

        let pekerja = thread::Builder::new()
            .name("waifujahat-shell-monitor".into())
            .spawn(move || {
                loop {
                    match pengirim.try_send(potret_proses()) {
                        Ok(()) | Err(TrySendError::Full(_)) => {}

                        Err(TrySendError::Disconnected(_)) => {
                            break;
                        }
                    }

                    match penerima_henti.recv_timeout(INTERVAL_PEMANTAUAN) {
                        Err(RecvTimeoutError::Timeout) => {}

                        _ => break,
                    }
                }
            })?;

        drop(pekerja);

        Ok(Self {
            penerima,
            _pengirim_henti: pengirim_henti,
        })
    }

    pub fn ambil(&self) -> Option<Result<Vec<ProsesTerminal>>> {
        self.penerima.try_recv().ok()
    }
}
