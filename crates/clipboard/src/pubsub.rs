use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

use crate::{ClipboardKind, ClipboardWait, Error};

type StateData = Mutex<(State, Option<mime::Mime>)>;

pub fn new(kind: ClipboardKind) -> (Publisher, Subscriber) {
    let inner = Arc::new((Mutex::new((State::Running, None)), Condvar::new()));
    let publisher = Publisher(inner.clone());
    let subscriber = Subscriber { inner, kind };
    (publisher, subscriber)
}

#[derive(Clone, Copy, Debug)]
enum State {
    Running,
    Stopped,
}

#[derive(Debug)]
pub struct Publisher(Arc<(StateData, Condvar)>);

impl Publisher {
    pub fn notify_all(&self, mime: mime::Mime) {
        let (lock, condvar) = &*self.0;
        *lock.lock() = (State::Running, Some(mime));
        let _unused = condvar.notify_all();
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let (lock, condvar) = &*self.0;
        *lock.lock() = (State::Stopped, None);
        let _unused = condvar.notify_all();
    }
}

#[derive(Clone, Debug)]
pub struct Subscriber {
    inner: Arc<(StateData, Condvar)>,
    kind: ClipboardKind,
}

// FIXME:
#[allow(clippy::significant_drop_in_scrutinee)]
impl ClipboardWait for Subscriber {
    fn wait(&self) -> Result<(ClipboardKind, mime::Mime), Error> {
        let (lock, condvar) = &*self.inner;
        let result = {
            let mut state = lock.lock();
            condvar.wait(&mut state);
            match *state {
                (State::Running, Some(ref mime)) => Ok((self.kind, mime.clone())),
                (State::Running | State::Stopped, _) => Err(Error::NotifierClosed),
            }
        };
        result
    }
}

impl Subscriber {
    #[inline]
    #[must_use]
    pub const fn clipboard_kind(&self) -> ClipboardKind { self.kind }
}
