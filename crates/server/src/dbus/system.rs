use zbus::dbus_interface;

pub struct SystemService {}

impl SystemService {
    #[inline]
    pub const fn new() -> Self { Self {} }
}

#[dbus_interface(name = "org.clipcat.clipcat.System")]
impl SystemService {
    #[allow(clippy::unused_self)]
    #[dbus_interface(property)]
    const fn get_version(&self) -> &str { clipcat_base::PROJECT_VERSION }
}
