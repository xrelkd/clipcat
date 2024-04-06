#![allow(clippy::ignored_unit_patterns)]

use std::{str::FromStr, sync::Arc};

use clipcat_dbus_variant as dbus_variant;
use tokio::sync::Mutex;
use zbus::interface;

use crate::{metrics, notification, ClipboardManager};

pub struct ManagerService<Notification> {
    manager: Arc<Mutex<ClipboardManager<Notification>>>,
}

impl<Notification> ManagerService<Notification> {
    pub fn new(manager: Arc<Mutex<ClipboardManager<Notification>>>) -> Self { Self { manager } }
}

#[interface(name = "org.clipcat.clipcat.Manager")]
impl<Notification> ManagerService<Notification>
where
    Notification: notification::Notification + 'static,
{
    async fn insert(&self, kind: dbus_variant::ClipboardKind, data: &[u8], mime: &str) -> u64 {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mime = mime::Mime::from_str(mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let mut manager = self.manager.lock().await;
        let id = manager.insert(
            clipcat_base::ClipEntry::new(data, &mime, kind.into(), None).unwrap_or_default(),
        );
        let _unused = manager.mark(id, kind.into()).await;
        drop(manager);
        id
    }

    #[zbus(property)]
    async fn set_clipboard_text_contents(&self, data: &str) {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let kind = clipcat_base::ClipboardKind::Clipboard;
        let mut manager = self.manager.lock().await;
        let id = manager.insert(
            clipcat_base::ClipEntry::new(data.as_bytes(), &mime::TEXT_PLAIN_UTF_8, kind, None)
                .unwrap_or_default(),
        );
        let _unused = manager.mark(id, kind).await;
        drop(manager);
    }

    #[zbus(property)]
    async fn clipboard_text_contents(&self) -> String {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let manager = self.manager.lock().await;
        manager
            .get_current_clip(clipcat_base::ClipboardKind::Clipboard)
            .map(clipcat_base::ClipEntry::as_utf8_string)
            .unwrap_or_default()
    }

    async fn remove(&self, id: u64) -> bool {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mut manager = self.manager.lock().await;
        manager.remove(id)
    }

    async fn batch_remove(&self, ids: Vec<u64>) -> Vec<u64> {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mut manager = self.manager.lock().await;
        ids.into_iter().filter(|&id| manager.remove(id)).collect()
    }

    async fn clear(&self) {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mut manager = self.manager.lock().await;
        manager.clear();
    }

    async fn get(&self, id: u64) -> zvariant::Optional<dbus_variant::ClipEntry> {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let manager = self.manager.lock().await;
        zvariant::Optional::from(manager.get(id).map(Into::into))
    }

    async fn get_current_clip(
        &self,
        kind: dbus_variant::ClipboardKind,
    ) -> zvariant::Optional<dbus_variant::ClipEntry> {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let manager = self.manager.lock().await;
        zvariant::Optional::from(
            manager.get_current_clip(kind.into()).map(|clip| clip.clone().into()),
        )
    }

    async fn list(&self, preview_length: u64) -> Vec<dbus_variant::ClipEntryMetadata> {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let manager = self.manager.lock().await;
        manager
            .list(usize::try_from(preview_length).unwrap_or(30))
            .into_iter()
            .map(dbus_variant::ClipEntryMetadata::from)
            .collect()
    }

    async fn update(&self, id: u64, data: &[u8], mime: &str) -> (bool, u64) {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let (ok, new_id) = {
            let mime = mime::Mime::from_str(mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            manager.replace(id, data, &mime)
        };
        (ok, new_id)
    }

    async fn mark(&self, id: u64, kind: dbus_variant::ClipboardKind) -> bool {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mut manager = self.manager.lock().await;
        manager.mark(id, kind.into()).await.is_ok()
    }

    #[zbus(property)]
    async fn length(&self) -> u64 {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let manager = self.manager.lock().await;
        manager.len() as u64
    }
}
