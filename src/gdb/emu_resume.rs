use gdbstub::common::Tid;
use gdbstub::target;

use crate::gdb::emulator::{EmulationCommand, Emulator, ExecutionMode};
use crate::plic::Fault;

impl target::ext::base::multithread::MultiThreadResume for Emulator {
    fn resume(&mut self) -> Result<(), Self::Error> {
        eprintln!("> resume");
        self.sender.send(EmulationCommand::Resume).expect("disco");
        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        eprintln!("> clear_resume_actions");
        self.sender
            .send(EmulationCommand::ClearResumeAction)
            .expect("disco");
        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        _tid: Tid,
        signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        if signal.is_some() {
            // No support for resuming via signals
            return Err(Fault::Unimplemented);
        }

        eprintln!("> set_resume_action_continue");
        self.sender
            .send(EmulationCommand::SetResumeAction(ExecutionMode::Continue))
            .expect("disco");
        Ok(())
    }
}
