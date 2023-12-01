use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

use crate::{ClipboardKind, ClipboardWait, Error};

pub fn new(kind: ClipboardKind) -> (Publisher, Subscriber) {
    let inner = Arc::new((Mutex::new(State::Running), Condvar::new()));
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
pub struct Publisher(Arc<(Mutex<State>, Condvar)>);

impl Publisher {
    pub fn notify_all(&self) {
        let (lock, condvar) = &*self.0;
        *lock.lock() = State::Running;
        let _unused = condvar.notify_all();
    }

    pub fn close(&self) {
        let (lock, condvar) = &*self.0;
        *lock.lock() = State::Stopped;
        let _unused = condvar.notify_all();
    }
}

impl Drop for Publisher {
    fn drop(&mut self) { self.close(); }
}

#[derive(Clone, Debug)]
pub struct Subscriber {
    inner: Arc<(Mutex<State>, Condvar)>,
    kind: ClipboardKind,
}

// FIXME:
#[allow(clippy::significant_drop_in_scrutinee)]
impl ClipboardWait for Subscriber {
    fn wait(&self) -> Result<ClipboardKind, Error> {
        let (lock, condvar) = &*self.inner;
        let result = {
            let mut state = lock.lock();
            condvar.wait(&mut state);
            match *state {
                State::Running => Ok(self.kind),
                State::Stopped => Err(Error::NotifierClosed),
            }
        };
        result
    }
}
