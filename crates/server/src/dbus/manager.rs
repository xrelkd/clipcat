#![allow(clippy::ignored_unit_patterns)]

use std::{str::FromStr, sync::Arc};

use clipcat_dbus_variant as dbus_variant;
use tokio::sync::Mutex;
use zbus::dbus_interface;

use crate::{notification, ClipboardManager};

pub struct ManagerService<Notification> {
    manager: Arc<Mutex<ClipboardManager<Notification>>>,
}

impl<Notification> ManagerService<Notification> {
    pub fn new(manager: Arc<Mutex<ClipboardManager<Notification>>>) -> Self { Self { manager } }
}

#[dbus_interface(name = "org.clipcat.clipcat.Manager")]
impl<Notification> ManagerService<Notification>
where
    Notification: notification::Notification + 'static,
{
    async fn insert(&self, kind: dbus_variant::ClipboardKind, data: &[u8], mime: &str) -> u64 {
        let mime = mime::Mime::from_str(mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let mut manager = self.manager.lock().await;
        let id = manager.insert(
            clipcat_base::ClipEntry::new(data, &mime, kind.into(), None).unwrap_or_default(),
        );
        let _unused = manager.mark(id, kind.into()).await;
        drop(manager);
        id
    }

    #[dbus_interface(property)]
    async fn set_clipboard_text_contents(&self, data: &str) {
        let kind = clipcat_base::ClipboardKind::Clipboard;
        let mut manager = self.manager.lock().await;
        let id = manager.insert(
            clipcat_base::ClipEntry::new(data.as_bytes(), &mime::TEXT_PLAIN_UTF_8, kind, None)
                .unwrap_or_default(),
        );
        let _unused = manager.mark(id, kind).await;
        drop(manager);
    }

    #[dbus_interface(property)]
    async fn clipboard_text_contents(&self) -> String {
        let manager = self.manager.lock().await;
        manager
            .get_current_clip(clipcat_base::ClipboardKind::Clipboard)
            .map(clipcat_base::ClipEntry::as_utf8_string)
            .unwrap_or_default()
    }

    async fn remove(&self, id: u64) -> bool {
        let mut manager = self.manager.lock().await;
        manager.remove(id)
    }

    async fn batch_remove(&self, ids: Vec<u64>) -> Vec<u64> {
        let mut manager = self.manager.lock().await;
        ids.into_iter().filter(|&id| manager.remove(id)).collect()
    }

    async fn clear(&self) {
        let mut manager = self.manager.lock().await;
        manager.clear();
    }

    async fn get(&self, id: u64) -> Option<dbus_variant::ClipEntry> {
        let manager = self.manager.lock().await;
        manager.get(id).map(Into::into)
    }

    async fn get_current_clip(
        &self,
        kind: dbus_variant::ClipboardKind,
    ) -> Option<dbus_variant::ClipEntry> {
        let manager = self.manager.lock().await;
        manager.get_current_clip(kind.into()).map(|clip| clip.clone().into())
    }

    async fn list(&self, preview_length: u64) -> Vec<dbus_variant::ClipEntryMetadata> {
        let manager = self.manager.lock().await;
        manager
            .list(usize::try_from(preview_length).unwrap_or(30))
            .into_iter()
            .map(dbus_variant::ClipEntryMetadata::from)
            .collect()
    }

    async fn update(&self, id: u64, data: &[u8], mime: &str) -> (bool, u64) {
        let (ok, new_id) = {
            let mime = mime::Mime::from_str(mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            manager.replace(id, data, &mime)
        };
        (ok, new_id)
    }

    async fn mark(&self, id: u64, kind: dbus_variant::ClipboardKind) -> bool {
        let mut manager = self.manager.lock().await;
        manager.mark(id, kind.into()).await.is_ok()
    }

    #[dbus_interface(property)]
    async fn length(&self) -> u64 {
        let manager = self.manager.lock().await;
        manager.len() as u64
    }
}
