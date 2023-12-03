use std::iter::IntoIterator;

use clipcat_base::ClipboardKind;
use clipcat_clipboard::ClipboardWait;
use tokio::{sync::mpsc, task};

#[derive(Debug)]
pub struct Subscriber {
    receiver: mpsc::UnboundedReceiver<(ClipboardKind, mime::Mime)>,
    join_handles: task::JoinSet<()>,
}

impl Subscriber {
    pub async fn next(&mut self) -> Option<(ClipboardKind, mime::Mime)> {
        self.receiver.recv().await
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.receiver.close();
        self.join_handles.abort_all();
    }
}

impl<I> From<I> for Subscriber
where
    I: IntoIterator<Item = clipcat_clipboard::Subscriber>,
{
    fn from(subs: I) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let join_handles =
            subs.into_iter().fold(task::JoinSet::new(), |mut join_handles, subscriber| {
                let _unused = join_handles.spawn_blocking({
                    let event_sender = sender.clone();
                    move || {
                        while let Ok(kind) = subscriber.wait() {
                            if event_sender.is_closed() {
                                break;
                            }

                            if let Err(_err) = event_sender.send(kind) {
                                break;
                            }
                        }
                    }
                });
                join_handles
            });

        Self { receiver, join_handles }
    }
}
