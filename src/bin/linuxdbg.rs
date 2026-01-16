use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::{env, fs};

use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError};
use gdbstub::stub::{DisconnectReason, GdbStub, SingleThreadStopReason};
use gdbstub::target::Target;
use log::{LevelFilter, error, info};
use log4rs::Config;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use object::{Object, ObjectSection};

use rriscv::bus::DynBus;
use rriscv::gdb::emu::{Emulator, StopReason};
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use rriscv::uart::Uart8250;
use rriscv::virtio::BlkDevice;
use rriscv::{clint, dt, plic};

/// Event loop implementation for blocking GDB stub
struct EmuEventLoop;

impl BlockingEventLoop for EmuEventLoop {
    type Target = Emulator;
    type Connection = TcpStream;
    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        Event<Self::StopReason>,
        WaitForStopReasonError<<Self::Target as Target>::Error, <Self::Connection as Connection>::Error>,
    > {
        loop {
            // Check for incoming GDB data (interrupt)
            match conn.peek() {
                Ok(Some(byte)) => {
                    // GDB sent something (likely Ctrl+C), signal incoming data
                    return Ok(Event::IncomingData(byte));
                }
                Ok(None) => {}
                Err(e) => return Err(WaitForStopReasonError::Connection(e)),
            }

            // Run the emulator for a batch of instructions
            let stop_reason = target.run();

            let gdb_stop_reason = match stop_reason {
                StopReason::Halted => SingleThreadStopReason::Terminated(Signal::SIGTERM),
                StopReason::Signal(sig) => SingleThreadStopReason::Signal(sig),
                StopReason::Breakpoint => SingleThreadStopReason::SwBreak(()),
                StopReason::DoneStep => SingleThreadStopReason::DoneStep,
                StopReason::Running => continue, // Keep running, check for GDB input
            };

            return Ok(Event::TargetStopped(gdb_stop_reason));
        }
    }

    fn on_interrupt(_target: &mut Self::Target) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        // When GDB sends Ctrl+C, return a signal stop
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = ConsoleAppender::builder().build();
    let rolling = CompoundPolicy::new(
        Box::new(SizeTrigger::new(5 * 1024 * 1024)),
        Box::new(FixedWindowRoller::builder().build("debug.log.{}", 3).unwrap()),
    );
    let debug = Appender::builder()
        .filter(Box::new(ThresholdFilter::new(LevelFilter::Debug)))
        .build(
            "riscv",
            Box::new(
                RollingFileAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("{d} {l}::{m}{n}")))
                    .build("debug.log", Box::new(rolling))?,
            ),
        );

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(debug)
        // Silence gdbstub's "Unknown command" INFO messages
        .logger(Logger::builder().build("gdbstub", LevelFilter::Warn))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();

    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let disk_file = args.get(2).expect("expect disc file");

    let bin_data = fs::read(image_file).expect("file");
    let elf = object::File::parse(&*bin_data).expect("parsing");

    let bus = Arc::new(DynBus::new());
    let ram = Ram::new();
    let pc = elf.entry() as usize;

    for section in elf.sections() {
        let name = section.name().expect("section name");
        if name.contains("data") || name.contains("text") {
            let start = section.address() as usize;
            if let Ok(data) = section.uncompressed_data() {
                ram.write(start - pc, data.to_vec());
            }
        }
    }

    let s = ram.size();
    bus.map(ram, pc..(pc + s));

    // Add low ram
    let ram = Ram::sized(0x10000);
    bus.map(ram, 0x0..0x10000);

    let rtc = Rtc::new();
    bus.map(rtc, 0x40000..0x40020);

    let console = Uart8250::new();
    bus.map(console, 0x10000000..0x10000010);

    // virtio block device vda
    let vda = BlkDevice::new(disk_file, bus.clone());
    bus.map(vda, 0x10001000..0x10002000);

    let clint = clint::Clint::new(bus.clone(), 0x40000);
    bus.map(clint, 0x2000000..0x2010000);

    let plic = plic::Plic::new();
    bus.map(plic, 0xc000000..0xc600000);

    let device_tree = dt::load("linux");
    let dtb_start = 0x80000;
    let dtb_end = dtb_start + device_tree.len();
    let dtb = Rom::new(device_tree);
    bus.map(dtb, 0x80000..dtb_end);

    let mut hart = Hart::new(0, pc, bus.clone());

    // linux register state
    hart.set_register(treg("a0"), 0);
    hart.set_register(treg("a1"), dtb_start as u64);
    hart.set_csr(rriscv::csr::SATP, 0);

    let listener = TcpListener::bind("127.0.0.1:9001").unwrap();
    info!("Listening on port 9001");

    let mut debugger = Emulator::new(hart);
    if let Ok((stream, _addr)) = listener.accept() {
        info!("Got connection");
        // Disable Nagle's algorithm for better responsiveness
        stream.set_nodelay(true)?;

        let gdb = GdbStub::new(stream);

        match gdb.run_blocking::<EmuEventLoop>(&mut debugger) {
            Ok(disconnect_reason) => match disconnect_reason {
                DisconnectReason::Disconnect => info!("GDB client disconnected"),
                DisconnectReason::TargetExited(code) => info!("Target exited with code {}", code),
                DisconnectReason::TargetTerminated(sig) => {
                    info!("Target terminated with signal {:?}", sig)
                }
                DisconnectReason::Kill => info!("GDB sent kill command"),
            },
            Err(e) => error!("GDB error: {:?}", e),
        }
    }
    info!("Connection closed");

    Ok(())
}
