use zbus::interface;

use crate::metrics;

pub struct SystemService {}

impl SystemService {
    #[inline]
    pub const fn new() -> Self { Self {} }
}

#[interface(name = "org.clipcat.clipcat.System")]
impl SystemService {
    #[allow(clippy::unused_self)]
    #[zbus(property)]
    fn get_version(&self) -> &str {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        clipcat_base::PROJECT_VERSION
    }
}
