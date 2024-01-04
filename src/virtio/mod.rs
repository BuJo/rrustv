use std::fmt::{Display, Formatter};

pub use self::blk::BlkDevice;

mod blk;

#[derive(Clone, Debug)]
struct Queue {
    ready: bool,
    size: u32,
    desc: usize,
    driver: usize,
    device: usize,
}

struct VirtqDesc {
    addr: usize,
    len: u32,
    flags: u16,
    next: u16,
}

impl VirtqDesc {
    const NEXT: u16 = 1;
    const WRITE: u16 = 2;
    const INDIRECT: u16 = 4;
}

impl Display for VirtqDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut flags = vec![];
        if self.flags & VirtqDesc::NEXT > 0 {
            flags.push("next");
        }
        if self.flags & VirtqDesc::WRITE > 0 {
            flags.push("write");
        }
        if self.flags & VirtqDesc::INDIRECT > 0 {
            flags.push("indirect");
        }
        write!(
            f,
            "virtq[0x{:x} {}] {:?} -> {}",
            self.addr, self.len, flags, self.next
        )
    }
}

struct VirtDescs<'a>(pub &'a Vec<VirtqDesc>);

impl<'a> Display for VirtDescs<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[",).expect(".");
        for desc in self.0 {
            let mut flags = vec![];
            if desc.flags & VirtqDesc::WRITE > 0 {
                flags.push("write");
            }
            if desc.flags & VirtqDesc::INDIRECT > 0 {
                flags.push("indirect");
            }
            write!(f, "virtq[0x{:x} {} {:?}], ", desc.addr, desc.len, flags).expect(".");
        }
        write!(f, "]",)
    }
}

#[allow(non_snake_case)]
struct State {
    DeviceFeatures: u64,
    DriverFeatures: u64,
    DriverFeaturesSel: Sel,
    DeviceFeaturesSel: Sel,
    status: u32,
    queue_idx: usize,
}

enum Sel {
    Low,
    High,
}

struct Status {}

#[allow(unused)]
impl Status {
    const ACKNOWLEDGE: u32 = 1;
    const DRIVER: u32 = 2;
    const FAILED: u32 = 128;
    const FEATURES_OK: u32 = 8;
    const DRIVER_OK: u32 = 4;
    const DEVICE_NEEDS_RESET: u32 = 64;
}

struct Register {}

#[allow(non_upper_case_globals)]
#[allow(unused)]
impl Register {
    const MagicValue: usize = 0x000;
    const Version: usize = 0x004;
    const DeviceID: usize = 0x008;
    const VendorID: usize = 0x00c;
    const DeviceFeatures: usize = 0x010;
    const DeviceFeaturesSel: usize = 0x014;
    const DriverFeatures: usize = 0x020;
    const DriverFeaturesSel: usize = 0x024;
    const QueueSel: usize = 0x030;
    const QueueSizeMax: usize = 0x034;
    const QueueSize: usize = 0x038;
    const QueueReady: usize = 0x044;
    const QueueNotify: usize = 0x050;
    const InterruptStatus: usize = 0x060;
    const InterruptACK: usize = 0x064;
    const Status: usize = 0x070;
    const QueueDescLow: usize = 0x080;
    const QueueDescHigh: usize = 0x084;
    const QueueDriverLow: usize = 0x090;
    const QueueDriverHigh: usize = 0x094;
    const QueueDeviceLow: usize = 0x0a0;
    const QueueDeviceHigh: usize = 0x0a4;
    const SHMSel: usize = 0x0ac;
    const SHMLenLow: usize = 0x0b0;
    const SHMLenHigh: usize = 0x0b4;
    const SHMBaseLow: usize = 0x0b7;
    const SHMBaseHigh: usize = 0x0bc;
    const QueueReset: usize = 0x0c0;
    const ConfigGeneration: usize = 0x0fc;
    const Config: usize = 0x100;
}

struct Features {}

#[allow(unused)]
impl Features {
    const VERSION_1: u32 = 32;
    const ACCESS_PLATFORM: u32 = 33;
}
