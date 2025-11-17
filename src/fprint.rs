use libfprint_rs::{FpContext, FpDevice, FpPrint};

pub fn init_reader() -> Option<FpDevice> {
    // Initialize fingerprint reader
    //
    let context = FpContext::new();
    let devices = context.devices();

    devices.first().expect("No fingerprint devices found")
}
