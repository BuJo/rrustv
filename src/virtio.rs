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
    queue_ready: bool,
    queue_idx: u32,
    queue_size: u32,
    queue_desc: u64,
    queue_driver: u64,
    queue_device: u64,
    features: u64,
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
    const MAX_DISCARD_SECTORS: usize = 0x24;
    const MAX_DISCARD_SEG: usize = 0x28;
    const DISCARD_SECTOR_ALIGNMENT: usize = 44;
    const MAX_WRITE_ZEROES_SECTORS: usize = 48;
    const MAX_WRITE_ZEROES_SEG: usize = 52;
    const WRITE_ZEROES_MAY_UNMAP: usize = 56;
    const MAX_SECURE_ERASE_SECTORS: usize = 60;
    const MAX_SECURE_ERASE_SEG: usize = 64;
    const SECURE_ERASE_SECTOR_ALIGNMENT: usize = 68;
    const ZONE_SECTORS: usize = 72;
    const MAX_OPEN_ZONES: usize = 76;
    const MAX_ACTIVE_ZONES: usize = 80;
    const MAX_APPEND_SECTORS: usize = 84;
    const WRITE_GRANULARITY: usize = 88;
    const MODEL: usize = 92;
}

struct Features {}

impl Features {
    const VERSION_1: u32 = 32;
    const ACCESS_PLATFORM: u32 = 33;
}

impl Device {
    pub fn new_block_device(_s: &str) -> Device {
        //let features = (1 << (Features::VERSION_1)) | (1 << (Features::ACCESS_PLATFORM));
        let features = (1 << (Features::VERSION_1));

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

                features: 0,
                status: 0,
                queue_idx: 0,

                // Queue 0
                queue_ready: false,
                queue_size: 0,
                queue_desc: 0,
                queue_driver: 0,
                queue_device: 0,
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
                    state.DriverFeatures = val as u64;
                    Ok(())
                }
                Sel::High => {
                    state.DriverFeatures = state.DriverFeatures | ((val as u64) << 32);
                    info!("virtio: selected driver features: {:b}", state.DriverFeatures);
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
            Register::QueueSel => {
                info!("virtio: selecting queue {}", val);
                state.queue_idx = val;
                Ok(())
            }
            Register::QueueReady => {
                info!(
                    "virtio: queue {}: setting ready: {}",
                    state.queue_idx,
                    val != 00
                );
                state.queue_ready = val != 0;
                Ok(())
            }
            Register::QueueSize => {
                info!("virtio: queue {}: setting size: {}", state.queue_idx, val);
                state.queue_size = val;
                Ok(())
            }
            Register::QueueDescLow => {
                state.queue_desc = val as u64;
                Ok(())
            }
            Register::QueueDescHigh => {
                info!(
                    "virtio: queue {}: setting descriptor area: {}",
                    state.queue_idx, val
                );
                state.queue_desc = ((val as u64) << 32) | state.queue_desc;
                Ok(())
            }
            Register::QueueDriverLow => {
                state.queue_driver = val as u64;
                Ok(())
            }
            Register::QueueDriverHigh => {
                info!(
                    "virtio: queue {}: setting driver area: {}",
                    state.queue_idx, val
                );
                state.queue_driver = ((val as u64) << 32) | state.queue_desc;
                Ok(())
            }
            Register::QueueDeviceLow => {
                state.queue_device = val as u64;
                Ok(())
            }
            Register::QueueDeviceHigh => {
                info!(
                    "virtio: queue {}: setting device area: {}",
                    state.queue_idx, val
                );
                state.queue_device = ((val as u64) << 32) | state.queue_desc;
                Ok(())
            }
            _ => Err(Fault::Unimplemented(format!(
                "virtio: writing register 0x{:x} unimplemented",
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
                "virtio: reading config register 0x{:x} unimplemented",
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
            Register::DeviceFeatures => {
                let features = (*state).DeviceFeatures;

                match (*state).DeviceFeaturesSel {
                    Sel::Low => Ok((features & 0xFFFFFFFF) as u32),
                    Sel::High => Ok((features >> 32) as u32),
                }
            }
            Register::Status => Ok(state.status),
            _ if addr >= 0x100 => {
                let addr = addr - 0x100;
                match addr {
                    BlkConfig::SIZE_MAX => Ok(512),
                    BlkConfig::SEG_MAX => Ok(1),
                    BlkConfig::BLK_SIZE => Ok(512),
                    BlkConfig::OPT_IO_SIZE => Ok(512),
                    BlkConfig::MAX_DISCARD_SECTORS => Ok(0),
                    BlkConfig::MAX_DISCARD_SEG => Ok(0),
                    BlkConfig::DISCARD_SECTOR_ALIGNMENT => Ok(0),
                    BlkConfig::MAX_WRITE_ZEROES_SECTORS => Ok(0),
                    BlkConfig::MAX_WRITE_ZEROES_SEG => Ok(0),
                    BlkConfig::MAX_SECURE_ERASE_SECTORS => Ok(0),
                    BlkConfig::MAX_SECURE_ERASE_SEG => Ok(0),
                    BlkConfig::SECURE_ERASE_SECTOR_ALIGNMENT => Ok(0),
                    BlkConfig::ZONE_SECTORS => Ok(0),
                    BlkConfig::MAX_OPEN_ZONES => Ok(0),
                    BlkConfig::MAX_ACTIVE_ZONES => Ok(0),
                    BlkConfig::MAX_APPEND_SECTORS => Ok(0),
                    BlkConfig::WRITE_GRANULARITY => Ok(0),
                    BlkConfig::MODEL => Ok(0),
                    _ => Err(Fault::Unimplemented(format!(
                        "virtio: reading config register 0x{:x}:u32 unimplemented",
                        addr
                    ))),
                }
            }
            Register::ConfigGeneration => Ok(0xdeadbeef),
            Register::QueueReady => Ok(state.queue_ready as u32),
            Register::QueueSizeMax => Ok(1),
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading register 0x{:x} unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u32 = {:?}", addr, res);

        res
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let addr = addr - 0x100;
        let res = match addr {
            BlkConfig::NUM_QUEUES => Ok(4),
            BlkConfig::MIN_IO_SIZE => Ok(1),
            BlkConfig::WRITE_ZEROES_MAY_UNMAP => Ok(0),
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading config register 0x{}:u16 unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u16 = {:?}", addr, res);

        res
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let addr = addr - 0x100;
        let res = match addr {
            BlkConfig::WRITEBACK => {
                Ok(0) // write through (1 is writeback)
            }
            BlkConfig::PHYSICAL_BLOCK_EXP => Ok(1), // one logical per physical block
            BlkConfig::ALIGNMENT_OFFSET => Ok(0),
            _ => Err(Fault::Unimplemented(format!(
                "virtio: reading config register 0x{:x}:u8 unimplemented",
                addr
            ))),
        };

        info!("virtio: reading 0x{:x}:u8 = {:?}", addr, res);

        res
    }
}
