use crossbeam::channel::{Receiver, Sender, TrySendError};

// Original by BlinkyStitt https://github.com/crossbeam-rs/crossbeam/issues/374#issuecomment-643378762

pub struct UnboundedBroadcast<T> {
    channels: Vec<Sender<T>>,
}

impl<T: 'static + Clone + Send + Sync> UnboundedBroadcast<T> {
    pub fn new() -> Self {
        Self { channels: vec![] }
    }

    /**
     * Creates a new subscriber channel and adds tx to the subscription list, and returns the corresponding rx.
     */
    pub fn subscribe(&mut self) -> Receiver<T> {
        let (tx, rx) = crossbeam::channel::unbounded();

        self.channels.push(tx);

        rx
    }

    //RAINY I could also have a copy that doesn't mutate on send fail, so it doesn't need mut self
    /**
     * Calls send on all subscribers.  Any that error (disconnected) are removed from the list.
     * Warning: blocks on each full channel, in turn, so one full channel near the start of
     * the list could block other channels further down from receiving the message.
     */
    pub fn send(&mut self, message: T) -> () {
        self.channels.retain(|c| {
            match c.send(message.clone()) {
                Ok(()) => return true,
                Err(_) => return false,
            };
        });
    }

    //DITTO
    /**
     * Calls try_send on all subscribers.  Any that error Disconnected are removed from the list.
     */
    pub fn try_send(&mut self, message: T) -> () {
        self.channels.retain(|c| {
            match c.try_send(message.clone()) {
                Ok(()) => return true,
                Err(TrySendError::Full(_)) => return true,
                Err(TrySendError::Disconnected(_)) => return false,
            };
        });
    }
}