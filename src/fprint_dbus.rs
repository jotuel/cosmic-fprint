use zbus::proxy;

#[proxy(
    interface = "net.reactivated.Fprint.Manager",
    default_service = "net.reactivated.Fprint",
    default_path = "/net/reactivated/Fprint/Manager"
)]
pub trait Manager {
    fn get_default_device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "net.reactivated.Fprint.Device",
    default_service = "net.reactivated.Fprint"
)]
pub trait Device {
    fn claim(&self, username: &str) -> zbus::Result<()>;
    fn release(&self) -> zbus::Result<()>;
    fn list_enrolled_fingers(&self, username: &str) -> zbus::Result<Vec<String>>;
    fn delete_enrolled_fingers(&self, username: &str) -> zbus::Result<()>;
    fn delete_enrolled_finger(&self, finger_name: &str) -> zbus::Result<()>;
    fn enroll_start(&self, finger_name: &str) -> zbus::Result<()>;
    fn enroll_stop(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn enroll_status(&self, result: String, done: bool) -> zbus::Result<()>;

    #[zbus(property, name = "num-enroll-stages")]
    fn num_enroll_stages(&self) -> zbus::Result<i32>;

    #[zbus(property, name = "scan-type")]
    fn scan_type(&self) -> zbus::Result<String>;
}
