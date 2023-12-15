use crate::notification::traits;

#[derive(Clone, Copy, Debug, Default)]
pub struct Notification {}

impl traits::Notification for Notification {}
