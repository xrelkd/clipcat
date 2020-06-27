use std::{collections::HashMap, pin::Pin};

use futures::Future;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::mpsc,
};

#[derive(Clone)]
pub struct ShutdownSignal(mpsc::Sender<()>);

pub struct ShutdownSlot(mpsc::Receiver<()>);

pub fn shutdown_handle() -> (ShutdownSignal, ShutdownSlot) {
    let (sender, receiver) = mpsc::channel(1);
    (ShutdownSignal(sender), ShutdownSlot(receiver))
}

impl ShutdownSignal {
    pub fn shutdown(self) { drop(self.0); }
}

impl ShutdownSlot {
    pub async fn wait(&mut self) { self.0.recv().await; }
}

pub type ShutdownHookFn = Box<dyn FnOnce() -> () + Send>;

pub struct LifecycleManager {
    shutdown_slot: ShutdownSlot,
    shutdown_hooks: HashMap<String, ShutdownHookFn>,
}

impl LifecycleManager {
    #[inline]
    pub fn new() -> (LifecycleManager, ShutdownSignal) {
        let (shutdown_signal, shutdown_slot) = shutdown_handle();
        (LifecycleManager { shutdown_hooks: HashMap::default(), shutdown_slot }, shutdown_signal)
    }

    #[inline]
    pub fn register(&mut self, name: &str, hook: ShutdownHookFn) {
        info!("shutdown hook registered [\"{}\"]", name);
        self.shutdown_hooks.insert(name.to_owned(), hook);
    }

    #[inline]
    pub async fn block_on<F>(self, fut: F) -> F::Output
    where
        F: futures::Future,
    {
        let signal_handle = self.prepare();

        tokio::spawn(signal_handle);
        fut.await
    }

    async fn prepare(self) {
        let shutdown_hooks = self.shutdown_hooks;
        let mut shutdown_slot = self.shutdown_slot;

        let signal_receiver = {
            type SignalFuture = Pin<Box<dyn Future<Output = ()> + Send>>;
            let mut signals: Vec<SignalFuture> = vec![];

            macro_rules! add_signal {
                ($e:expr) => {
                    signals.push(Box::pin(async move { $e }));
                };
            };

            add_signal!({
                let mut term_signal = signal(SignalKind::terminate()).unwrap();
                term_signal.recv().await;
                info!("SIGTERM received!");
            });

            add_signal!({
                let mut int_signal = signal(SignalKind::interrupt()).unwrap();
                int_signal.recv().await;
                info!("SIGINT received!");
            });

            add_signal!({
                shutdown_slot.wait().await;
                info!("Internal shutdown signal received!");
            });

            futures::future::select_all(signals)
        };

        info!("Waiting for shutdown signal...");
        let _ = signal_receiver.await;

        for (name, hook) in shutdown_hooks {
            info!("Trigger registered hook [\"{}\"]", name);
            hook();
        }
    }
}
