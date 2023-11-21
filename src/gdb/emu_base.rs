use std::sync::mpsc;

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target;
use gdbstub::target::TargetResult;

use crate::gdb::emulator::{EmulationCommand, Emulator};

impl target::ext::base::multithread::MultiThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let (sender, receiver) = mpsc::channel();
        self.sender
            .send(EmulationCommand::ReadRegisters(sender))
            .expect("disco");
        let registers = receiver.recv().expect("disco");
        regs.pc = registers[0];
        for i in 0..=31 {
            regs.x[i] = registers[i + 1];
        }

        eprintln!("reading registers from tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let mut registers = vec![regs.pc];
        registers.extend_from_slice(&regs.x);
        self.sender
            .send(EmulationCommand::SetRegisters(registers))
            .expect("disco");

        eprintln!("writing registers to tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.bus.read(start_addr as usize, data).unwrap_or_default();

        eprintln!("reading from tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &[u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.bus.write(start_addr as usize, data).expect("asdf");

        eprintln!("writing to tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), Self::Error> {
        eprintln!("registering active thread: {}", 1);
        thread_is_active(Tid::new(1).unwrap());
        Ok(())
    }

    fn support_resume(
        &mut self,
    ) -> Option<target::ext::base::multithread::MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }
}
