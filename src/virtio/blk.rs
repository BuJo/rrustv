use std::fs::File;
use std::os::unix::fs::FileExt;
use std::sync::{Arc, RwLock};

use log::{info, trace};

use crate::bus::DynBus;
use crate::device::Device;
use crate::irq::Interrupt;
use crate::virtio::{Features, Queue, Register, Sel, State, Status, VirtDescs, VirtqDesc};

#[allow(non_snake_case)]
pub struct BlkDevice {
    MagicValue: u32, // R
    Version: u32,    // R
    DeviceID: u32,
    VendorID: u32,

    bus: Arc<DynBus>,
    file: RwLock<File>,
    capacity: u64,

    state: RwLock<State>,
    queues: RwLock<Vec<Queue>>,
}

struct BlkFlag {}

#[allow(unused)]
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
    const CAPACITY_HIGH: usize = 4;
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

#[derive(Debug)]
struct RequestHeader {
    typ: RequestType,
    _ioprio: u32,
    sector_num: u64,
}

#[derive(Debug)]
enum RequestType {
    In = 0,
    Out = 1,
}

impl TryFrom<u32> for RequestType {
    type Error = Interrupt;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RequestType::In),
            1 => Ok(RequestType::Out),
            _ => Err(Interrupt::Unimplemented("unknown request type".into())),
        }
    }
}

impl BlkDevice {
    const MAX_QUEUES: usize = 16;

    pub fn new(s: &str, bus: Arc<DynBus>) -> BlkDevice {
        let features = (1 << (Features::VERSION_1))
            | (1 << (BlkFlag::SIZE_MAX))
            | (1 << (BlkFlag::SEG_MAX))
            | (1 << (BlkFlag::RO))
            | (1 << (BlkFlag::BLK_SIZE));

        let file = File::open(s).expect("file being there");
        let file_bytes = file.metadata().unwrap().len();
        let capacity = (file_bytes / 512) + 1; // TODO: incorrect capacity computation

        BlkDevice {
            MagicValue: 0x74726976, // little endian "virt"
            Version: 0x2,           // non-legacy blk version
            DeviceID: 2,            // block device
            VendorID: 0x1af4,       // emulated

            file: RwLock::new(file),

            bus,
            capacity,

            state: RwLock::new(State {
                DeviceFeatures: features,
                DriverFeatures: 0,
                DeviceFeaturesSel: Sel::Low,
                DriverFeaturesSel: Sel::Low,

                status: 0,
                queue_idx: 0,
            }),
            queues: RwLock::new(vec![
                Queue {
                    ready: false,
                    size: 0,
                    desc: 0,
                    driver: 0,
                    device: 0,
                };
                BlkDevice::MAX_QUEUES
            ]),
        }
    }

    fn get_desc_rw_size(&self, queue: &Queue, mut desc_idx: u16) -> (u32, u32) {
        let mut read_size = 0;
        let mut write_size = 0;

        loop {
            let desc = self.get_desc(queue, desc_idx);

            if desc.flags & VirtqDesc::WRITE > 0 {
                write_size += desc.len;
            } else {
                read_size += desc.len;
            }

            if desc.flags & VirtqDesc::NEXT > 0 {
                break;
            }
            desc_idx = desc.next;
        }

        (read_size, write_size)
    }

    fn recv_request(&self, queue: &Queue, head_idx: u16) {
        let hdr_desc = self.get_desc(queue, head_idx);

        let req = RequestHeader {
            typ: RequestType::try_from(self.bus.read_word(hdr_desc.addr).unwrap()).unwrap(),
            _ioprio: self.bus.read_word(hdr_desc.addr + 4).unwrap(),
            sector_num: self.bus.read_double(hdr_desc.addr + 8).unwrap(),
        };

        info!(
            "request {} header 0x{:x} says: {:?}",
            head_idx, hdr_desc.addr, req
        );

        let buf_desc = self.get_desc(queue, hdr_desc.next);

        match req.typ {
            RequestType::In => {
                // driver wants to read data
                let file = self.file.write().unwrap();
                let mut space = vec![0; buf_desc.len as usize];
                file.read_exact_at(&mut space, req.sector_num * 512)
                    .unwrap();
                for (i, b) in space.iter().enumerate() {
                    self.bus.write_byte(buf_desc.addr + i, *b).unwrap();
                }
                info!(
                    "request {} read sector {} (byte offset {}) len {} from block device",
                    head_idx,
                    req.sector_num,
                    req.sector_num * 512,
                    buf_desc.len as usize
                );
            }
            RequestType::Out => {
                // driver wants to write data
                info!(
                    "request {} writing sector {} from block device",
                    head_idx, req.sector_num
                );
            }
        }

        let status_desc = self.get_desc(queue, hdr_desc.next);
        let status = self.bus.read_byte(status_desc.addr).unwrap();
        info!("request {} status {}:", head_idx, status);
    }

    fn get_desc(&self, queue: &Queue, desc_idx: u16) -> VirtqDesc {
        let addr = queue.desc + 16 * desc_idx as usize;
        VirtqDesc {
            addr: self.bus.read_double(addr).unwrap() as usize,
            len: self.bus.read_word(addr + 8).unwrap(),
            flags: self.bus.read_half(addr + 12).unwrap(),
            next: self.bus.read_half(addr + 14).unwrap(),
        }
    }
}

impl Device for BlkDevice {
    fn write_double(&self, _addr: usize, _val: u64) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing double unimplemented".into(),
        ))
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Interrupt> {
        trace!("writing 0x{:x} = {}", addr, val);

        let mut state = self.state.write().unwrap();
        let mut queues = self.queues.write().unwrap();

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
            Register::DriverFeatures => match state.DriverFeaturesSel {
                Sel::Low => {
                    state.DriverFeatures = val as u64;
                    Ok(())
                }
                Sel::High => {
                    state.DriverFeatures |= (val as u64) << 32;
                    info!("selected driver features: {:b}", state.DriverFeatures);
                    Ok(())
                }
            },
            Register::Status => {
                if val == 0 {
                    info!("initializing device");
                }
                if val & Status::ACKNOWLEDGE > 0 {
                    info!("driver acked");
                    state.status |= Status::ACKNOWLEDGE;
                }
                if val & Status::DRIVER > 0 {
                    info!("driver is indeed a driver");
                    state.status |= Status::DRIVER;
                }
                if val & Status::FEATURES_OK > 0 {
                    info!("driver likes the devices features");
                    state.status |= Status::FEATURES_OK;
                }
                if val & Status::DRIVER_OK > 0 {
                    info!("driver likes the device");
                    state.status |= Status::DRIVER_OK;
                }
                if val & Status::DEVICE_NEEDS_RESET > 0 {
                    info!("driver needs the device to reset");
                    state.status = 0;
                }
                if val & Status::FAILED > 0 {
                    info!("driver thinks the device is a failure");
                }
                Ok(())
            }
            Register::QueueSel => {
                info!("selecting queue {}", val);
                state.queue_idx = val as usize;
                Ok(())
            }
            Register::QueueReady => {
                info!("queue {}: setting ready: {}", state.queue_idx, val != 0);
                queues[state.queue_idx].ready = val != 0;
                Ok(())
            }
            Register::QueueSize => {
                info!("queue {}: setting size: {}", state.queue_idx, val);
                queues[state.queue_idx].size = val;
                Ok(())
            }
            Register::QueueNotify => {
                // notifies that there are new buffers set up to process in the queue
                let idx = val;
                let queue = &queues[idx as usize];
                let mut addr = queue.desc;

                let mut descriptors: Vec<VirtqDesc> = vec![];
                loop {
                    let desc = VirtqDesc {
                        addr: self.bus.read_double(addr).unwrap() as usize,
                        len: self.bus.read_word(addr + 8).unwrap(),
                        flags: self.bus.read_half(addr + 12).unwrap(),
                        next: self.bus.read_half(addr + 14).unwrap(),
                    };
                    let next = desc.next as usize;
                    descriptors.push(desc);
                    if next == 0 {
                        break;
                    }
                    addr += 16 * next;
                }

                info!(
                    "queue {} to process: {:?}: {}",
                    idx,
                    queue,
                    VirtDescs(&descriptors)
                );

                // Avail == driver queue.  Device must only read.
                let baddr = queue.driver;
                let bflags = self.bus.read_half(baddr).unwrap();
                let bidx = self.bus.read_half(baddr + 2).unwrap();
                let mut ring_contents = vec![];
                for i in 0..queue.size {
                    let ring = self.bus.read_half(baddr + 4 + (i as usize)).unwrap();
                    ring_contents.push(ring);
                }

                info!(
                    "queue {} avail: {} 0b{:016b}: {:?}",
                    idx, bflags, bidx, ring_contents
                );

                // Used == device queue.  Device can write.
                let baddr = queue.device;
                let bflags = self.bus.read_half(baddr).unwrap();
                let bidx = self.bus.read_half(baddr + 2).unwrap();
                let mut ring_contents = vec![];
                for i in 0..queue.size {
                    let ring = self.bus.read_half(baddr + 4 + (i as usize)).unwrap();
                    ring_contents.push(ring);
                }

                info!(
                    "queue {} used: {} 0b{:016b}: {:?}",
                    idx, bflags, bidx, ring_contents
                );

                // XXX: lying, current index might be different
                let mut current_idx = 0;
                let avail_idx = self.bus.read_half(queue.driver + 2).unwrap();
                while current_idx != avail_idx {
                    // index of head descriptor for current item
                    let head_idx = self
                        .bus
                        .read_half(
                            queue.driver
                                + 4
                                + (current_idx as usize & (queue.size as usize - 1)) * 2,
                        )
                        .unwrap();
                    let (read_size, write_size) = self.get_desc_rw_size(queue, head_idx);
                    info!(
                        "queue {} ({}->{}) read: {} bytes, write {} bytes",
                        idx, current_idx, avail_idx, read_size, write_size
                    );

                    self.recv_request(queue, head_idx);
                    current_idx += 1;
                }

                Ok(())
            }
            Register::QueueDescLow => {
                queues[state.queue_idx].desc = val as usize;
                Ok(())
            }
            Register::QueueDescHigh => {
                queues[state.queue_idx].desc |= (val as usize) << 32;
                let addr = queues[state.queue_idx].desc;

                let desc = VirtqDesc {
                    addr: self.bus.read_double(addr).unwrap() as usize,
                    len: self.bus.read_word(addr + 8).unwrap(),
                    flags: self.bus.read_half(addr + 12).unwrap(),
                    next: self.bus.read_half(addr + 14).unwrap(),
                };

                info!(
                    "queue {}: setting descriptor area: 0x{:x}: {}",
                    state.queue_idx, addr, desc,
                );
                Ok(())
            }
            Register::QueueDriverLow => {
                queues[state.queue_idx].driver = val as usize;
                Ok(())
            }
            Register::QueueDriverHigh => {
                queues[state.queue_idx].driver |= (val as usize) << 32;

                info!(
                    "queue {}: setting driver area: 0x{:x}",
                    state.queue_idx, queues[state.queue_idx].driver
                );
                Ok(())
            }
            Register::QueueDeviceLow => {
                queues[state.queue_idx].device = val as usize;
                Ok(())
            }
            Register::QueueDeviceHigh => {
                queues[state.queue_idx].device |= (val as usize) << 32;
                info!(
                    "queue {}: setting device area: 0x{:x}",
                    state.queue_idx, queues[state.queue_idx].device
                );
                Ok(())
            }
            _ => Err(Interrupt::Unimplemented(format!(
                "writing register 0x{:x} unimplemented",
                addr
            ))),
        }
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        let addr = addr - 0x100;

        match addr {
            BlkConfig::CAPACITY => Ok(1),
            _ => Err(Interrupt::Unimplemented(format!(
                "reading config register 0x{:x} unimplemented",
                addr
            ))),
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        let state = self.state.read().unwrap();
        let queues = self.queues.write().unwrap();

        match addr {
            Register::MagicValue => Ok(self.MagicValue),
            Register::Version => Ok(self.Version),
            Register::DeviceID => Ok(self.DeviceID),
            Register::VendorID => Ok(self.VendorID),
            Register::DeviceFeatures => {
                let features = state.DeviceFeatures;

                match state.DeviceFeaturesSel {
                    Sel::Low => Ok((features & 0xFFFFFFFF) as u32),
                    Sel::High => Ok((features >> 32) as u32),
                }
            }
            Register::Status => Ok(state.status),
            _ if addr >= 0x100 => {
                let addr = addr - 0x100;
                match addr {
                    BlkConfig::CAPACITY => Ok((self.capacity & 0xFFFFFFFF) as u32),
                    BlkConfig::CAPACITY_HIGH => Ok((self.capacity >> 32) as u32),
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
                    _ => Err(Interrupt::Unimplemented(format!(
                        "reading config register 0x{:x}:u32 unimplemented",
                        addr
                    ))),
                }
            }
            Register::ConfigGeneration => Ok(0xdeadbeef),
            Register::QueueReady => Ok(queues[state.queue_idx].ready as u32),
            Register::QueueSizeMax => Ok(queues.capacity() as u32),
            _ => Err(Interrupt::Unimplemented(format!(
                "reading register 0x{:x} unimplemented",
                addr
            ))),
        }
    }

    fn read_half(&self, addr: usize) -> Result<u16, Interrupt> {
        let addr = addr - 0x100;
        match addr {
            BlkConfig::NUM_QUEUES => Ok(4),
            BlkConfig::MIN_IO_SIZE => Ok(1),
            BlkConfig::WRITE_ZEROES_MAY_UNMAP => Ok(0),
            _ => Err(Interrupt::Unimplemented(format!(
                "reading config register 0x{}:u16 unimplemented",
                addr
            ))),
        }
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Interrupt> {
        let addr = addr - 0x100;
        let res = match addr {
            BlkConfig::WRITEBACK => {
                Ok(0) // write through (1 is writeback)
            }
            BlkConfig::PHYSICAL_BLOCK_EXP => Ok(1), // one logical per physical block
            BlkConfig::ALIGNMENT_OFFSET => Ok(0),
            _ => Err(Interrupt::Unimplemented(format!(
                "reading config register 0x{:x}:u8 unimplemented",
                addr
            ))),
        };

        info!("reading 0x{:x}:u8 = {:?}", addr, res);

        res
    }
}
