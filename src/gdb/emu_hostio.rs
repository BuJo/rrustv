use std::cmp::min;
use std::fs::File;
use std::os::fd::{FromRawFd, IntoRawFd, RawFd};
use std::os::unix::fs::FileExt;

use gdbstub::common::Pid;
use gdbstub::target::ext::host_io::{
    HostIoCloseOps, HostIoError, HostIoOpenFlags, HostIoOpenMode, HostIoOpenOps, HostIoPreadOps,
    HostIoResult,
};
use gdbstub::target::{TargetError, TargetResult};

use crate::gdb::emulator::Emulator;
use crate::plic::Fault::{Unaligned, Unimplemented};

impl gdbstub::target::ext::host_io::HostIo for Emulator {
    fn support_open(&mut self) -> Option<HostIoOpenOps<'_, Self>> {
        Some(self)
    }

    fn support_close(&mut self) -> Option<HostIoCloseOps<'_, Self>> {
        Some(self)
    }

    fn support_pread(&mut self) -> Option<HostIoPreadOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::host_io::HostIoOpen for Emulator {
    fn open(
        &mut self,
        filename: &[u8],
        _flags: HostIoOpenFlags,
        _mode: HostIoOpenMode,
    ) -> HostIoResult<u32, Self> {
        eprintln!("{}", String::from_utf8(filename.into()).unwrap());
        Ok(File::open(String::from_utf8(Vec::from(filename)).unwrap())
            .unwrap()
            .into_raw_fd() as u32)
    }
}
impl gdbstub::target::ext::host_io::HostIoClose for Emulator {
    fn close(&mut self, fd: u32) -> HostIoResult<(), Self> {
        // Safety:
        // to close, we must acquire the fd from the open call.
        unsafe { File::from_raw_fd(fd as RawFd) };
        Ok(())
    }
}
impl gdbstub::target::ext::host_io::HostIoPread for Emulator {
    fn pread(
        &mut self,
        fd: u32,
        count: usize,
        offset: u64,
        buf: &mut [u8],
    ) -> HostIoResult<usize, Self> {
        let file = unsafe { File::from_raw_fd(fd as RawFd) };

        let len = file.read_at(buf, offset);

        // borrow file again
        file.into_raw_fd();

        len.map_err(|x| HostIoError::Fatal(Unaligned(offset as usize)))
    }
}

impl gdbstub::target::ext::exec_file::ExecFile for Emulator {
    fn get_exec_file(
        &self,
        _pid: Option<Pid>,
        offset: u64,
        length: usize,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        if self.exec_file.len() == 0 {
            return Err(TargetError::Fatal(Unimplemented));
        }
        let offset = offset as usize;
        if offset > self.exec_file.len() {
            return Ok(0);
        }
        let end = min(offset + length, self.exec_file.len());
        let len = end - offset;
        buf[..len].copy_from_slice(&self.exec_file[offset..end]);
        eprintln!("{}", String::from_utf8(buf[..len].into()).unwrap());
        Ok(self.exec_file.len())
    }
}
