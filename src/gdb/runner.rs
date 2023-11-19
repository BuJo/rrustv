use std::net::TcpStream;

use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::{MultiThreadStopReason, run_blocking};
use gdbstub::target::Target;

use crate::gdb::emulator::Emulator;

pub(crate) enum GdbBlockingEventLoop {}

impl run_blocking::BlockingEventLoop for GdbBlockingEventLoop {
    type Target = Emulator;
    type Connection = TcpStream;
    type StopReason = MultiThreadStopReason<u64>;

    // Invoked immediately after the target's `resume` method has been
    // called. The implementation should block until either the target
    // reports a stop reason, or if new data was sent over the connection.
    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        Ok(target.read_stop_event())
    }

    // Invoked when the GDB client sends a Ctrl-C interrupt.
    fn on_interrupt(
        target: &mut Self::Target,
    ) -> Result<Option<MultiThreadStopReason<u64>>, <Self::Target as Target>::Error> {
        // notify the target that a ctrl-c interrupt has occurred.
        target.stop_in_response_to_ctrl_c_interrupt()?;

        // a pretty typical stop reason in response to a Ctrl-C interrupt is to
        // report a "Signal::SIGINT".
        Ok(Some(MultiThreadStopReason::Signal(Signal::SIGINT).into()))
    }
}
