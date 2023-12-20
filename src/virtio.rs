use std::sync::RwLock;

use log::info;

use crate::device::Device as D;
use crate::plic::Fault;

#[allow(non_snake_case)]
pub struct Device {
    MagicValue: u32, // R
    Version: u32,    // R
    DeviceID: u32,
    VendorID: u32,

    state: RwLock<State>,
}

#[allow(non_snake_case)]
struct State {
    DeviceFeatures: u64,
    DriverFeatures: u64,
    DriverFeaturesSel: Sel,
    DeviceFeaturesSel: Sel,
    status: u32,
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

struct BlkFlag {}

impl BlkFlag {
    const SIZE_MAX: u32 = 1;
    const SEG_MAX: u32 = 2;
    const RO: u32 = 5;
    const BLK_SIZE: u32 = 6;
}

struct BlkConfig {}

#[allow(unused)]
impl BlkConfig {
    const CAPACITY: usize = 0;
    const SIZE_MAX: usize = 8;
    const SEG_MAX: usize = 12;
    const CYLINDERS: usize = 16;
    const HEADS: usize = 18;
    const SECTORS: usize = 19;
    const BLK_SIZE: usize = 20;
    const PHYSICAL_BLOCK_EXP: usize = 24;
    const ALIGNMENT_OFFSET: usize = 25;
    const MIN_IO_SIZE: usize = 26;
    const OPT_IO_SIZE: usize = 28;
    const WRITEBACK: usize = 32;
    const NUM_QUEUES: usize = 34;
}

struct Features {}

impl Features {
    const VERSION_1: u32 = 32;
    const ACCESS_PLATFORM: u32 = 33;
}

impl Device {
    pub fn new_block_device(_s: &str) -> Device {
        let features = (1 << (Features::VERSION_1 - 1))
            | (1 << (Features::ACCESS_PLATFORM - 1))
            | (1 << (BlkFlag::SIZE_MAX - 1))
            | (1 << (BlkFlag::SEG_MAX - 1))
            | (1 << (BlkFlag::RO - 1))
            | (1 << (BlkFlag::BLK_SIZE - 1));

        Device {
            MagicValue: 0x74726976, // little endian "virt"
            Version: 0x2,           // non-legacy virtio version
            DeviceID: 2,            // block device
            VendorID: 0x1af4,       // emulated

            state: RwLock::new(State {
                DeviceFeatures: features,
                DriverFeatures: 0,
                DeviceFeaturesSel: Sel::Low,
                DriverFeaturesSel: Sel::Low,
                status: 0,
            }),
        }
    }
}

impl D for Device {
    fn write_double(&self, _addr: usize, _val: u64) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "virtio: writing double unimplemented".into(),
        ))
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        info!("virtio: writing 0x{:x} = {}", addr, val);

        let mut state = self.state.write().unwrap();

        match addr {
            Register::DeviceFeaturesSel => {
                state.DeviceFeaturesSel = match val {
                    1 => Sel::High,
                    _ => Sel::Low,
                };
                Ok(())
            }
            Register::DriverFeaturesSel => {
                state.DriverFeaturesSel = match val {
                    1 => Sel::High,
                    _ => Sel::Low,
                };
                Ok(())
            }
            Register::DriverFeatures => match (*state).DriverFeaturesSel {
                Sel::Low => {
                    state.DriverFeatures =
                        state.DriverFeatures & 0xFFFFFFFF_00000000 | (val as u64);
                    Ok(())
                }
                Sel::High => {
                    state.DriverFeatures =
                        state.DriverFeatures & 0x00000000_FFFFFFFF | ((val as u64) << 32);
                    Ok(())
                }
            },
            Register::Status => {
                if val == 0 {
                    info!("virtio: initializing device");
                }
                if val & Status::ACKNOWLEDGE == Status::ACKNOWLEDGE {
                    info!("virtio: driver acked");
                    state.status |= Status::ACKNOWLEDGE;
                }
                if val & Status::DRIVER == Status::DRIVER {
                    info!("virtio: driver is indeed a driver");
                    state.status |= Status::DRIVER;
                }
                if val & Status::FEATURES_OK == Status::FEATURES_OK {
                    info!("virtio: driver likes the devices features");
                    state.status |= Status::FEATURES_OK;
                }
                if val & Status::DRIVER_OK == Status::DRIVER_OK {
                    info!("virtio: driver likes the device");
                    state.status |= Status::DRIVER_OK;
                }
                if val & Status::DEVICE_NEEDS_RESET == Status::DEVICE_NEEDS_RESET {
                    info!("virtio: driver needs the device to reset");
                    state.status = 0;
                }
                if val & Status::FAILED == Status::FAILED {
                    info!("virtio: driver thinks the device is a failure");
                }
                Ok(())
            }
            _ => Err(Fault::Unimplemented(format!(
                "virtio: writing register {} unimplemented",
                addr
            ))),
        }
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "virtio: writing halfword unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "virtio: writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let addr = addr - 0x100;
        let res = match addr {
            BlkConfig::CAPACITY => Ok(1),
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading config register {} unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u64 = {:?}", addr, res);

        res
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let state = self.state.read().unwrap();

        let res = match addr {
            Register::MagicValue => Ok(self.MagicValue),
            Register::Version => Ok(self.Version),
            Register::DeviceID => Ok(self.DeviceID),
            Register::VendorID => Ok(self.VendorID),
            Register::DeviceFeatures => match (*state).DeviceFeaturesSel {
                Sel::Low => Ok(((*state).DeviceFeatures | 0xFFFFFFFF) as u32),
                Sel::High => Ok(((*state).DeviceFeatures >> 32) as u32),
            },
            Register::Status => Ok(state.status),
            _ if addr >= 0x100 => {
                let addr = addr - 0x100;
                match addr {
                    BlkConfig::SIZE_MAX => Ok(512),
                    BlkConfig::SEG_MAX => Ok(1),
                    _ => Err(Fault::Unimplemented(format!(
                        "virtio: reading config register {} unimplemented",
                        addr
                    ))),
                }
            }
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading register {} unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u32 = {:?}", addr, res);

        res
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let addr = addr - 0x100;
        let res = match addr {
            BlkConfig::NUM_QUEUES => Ok(1),
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading config register {} unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u16 = {:?}", addr, res);

        res
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let addr = addr - 0x100;
        let res = match addr {
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading config register {} unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u8 = {:?}", addr, res);

        res
    }
}
