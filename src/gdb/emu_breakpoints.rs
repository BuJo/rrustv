use gdbstub::arch::Arch;
use gdbstub::target;
use gdbstub::target::TargetResult;

use crate::gdb::emulator::{EmulationCommand, Emulator};

impl target::ext::breakpoints::Breakpoints for Emulator {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_breakpoint(
        &mut self,
    ) -> Option<target::ext::breakpoints::HwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::breakpoints::SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("adding software breakpoint on {:x}({})", addr, kind);

        self.sender
            .send(EmulationCommand::AddBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("removing software breakpoint on {:x}({})", addr, kind);

        self.sender
            .send(EmulationCommand::RemoveBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }
}

impl target::ext::breakpoints::HwBreakpoint for Emulator {
    fn add_hw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("adding hardware breakpoint on {:x}({})", addr, kind);

        self.sender
            .send(EmulationCommand::AddBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }

    fn remove_hw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!(
            "removing hardware breakpoitarget remote localhost:9001nt on {:x}({})",
            addr, kind
        );

        self.sender
            .send(EmulationCommand::RemoveBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }
}
