use std::thread;

use crossbeam::channel::{Sender, Receiver};

pub struct AutodropThreadToken {
    exit: Sender<()>,
    exit_confirm: Receiver<()>,
    block_drop: bool,
}

impl AutodropThreadToken {
    /**
     * Spawns a new thread.  Keep its token alive for however long you want the thread to exist.
     * When its associated token is dropped, () is sent over the channel passed into the closure.
     * Monitor the closure so you know when your thread should exit.
     * `block_drop` indicates whether `drop` (and therefore your code) should be blocked until the
     * closure exits.  Understand this could deadlock code that looks unrelated, if the closure
     * never checks for the exit notification, or fails to exit after receiving it.
     * 
     * Note: how you store the token matters.  By brief testing:
     * Drops immediately: let _ = AutodropThreadToken::spawn()
     * Drops at the end of the scope, even if unused: let _foo = AutodropThreadToken::spawn()
     * Drops at the end of the scope, even if unused: let foo = AutodropThreadToken::spawn()
     * 
     * //THINK Implement extended rendezvous?
     */
    pub fn spawn<F>(block_drop: bool, f: F) -> AutodropThreadToken
    where
        F: FnOnce(Receiver<()>),
        F: Send + 'static,
    {
        let (exit_tx, exit_rx) = if block_drop {
            crossbeam::channel::bounded::<()>(0)
        } else {
            crossbeam::channel::bounded::<()>(1)
        };
        let (exit_conf_tx, exit_conf_rx) = crossbeam::channel::bounded::<()>(0);

        let t = AutodropThreadToken {
            exit: exit_tx,
            exit_confirm: exit_conf_rx,
            block_drop,
        };
        thread::spawn(move || {
            f(exit_rx);
            exit_conf_tx.send(()).ok(); // Discarding error
        });

        return t;
    }
}

impl Drop for AutodropThreadToken {
    fn drop(&mut self) {
        self.exit.send(()).ok(); // Discarding error
        if self.block_drop {
            match self.exit_confirm.recv() {
                Ok(()) => (),
                Err(_) => (), // Discarding error
            }
        }
    }
}