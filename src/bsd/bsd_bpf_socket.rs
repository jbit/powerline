extern crate std;

use crate::*;
use libc::{bpf_hdr, c_uint, suseconds_t, time_t, timeval};
use libc::{close, ioctl, open, read, write};
use libc::{BIOCGSTATS, BIOCIMMEDIATE, BIOCSBLEN, BIOCSETIF, BIOCSRTIMEOUT};
use libc::{BPF_ALIGNMENT, EBUSY, IF_NAMESIZE, O_RDWR};
use std::ffi::{c_void, CStr, CString};
use std::fs::read_dir;
use std::io::{Error, ErrorKind, Result};
use std::mem::size_of_val;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::RawFd;
use std::vec::Vec;
use std::{borrow::ToOwned, time::Instant};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Default)]
struct bpf_stat {
    packets_received: c_uint,
    packets_dropped: c_uint,
}

#[repr(align(16))]
struct BpfBuffer([u8; BpfBuffer::SIZE]);
impl BpfBuffer {
    const SIZE: usize = 4096;
    pub fn new() -> BpfBuffer {
        BpfBuffer([0u8; BpfBuffer::SIZE])
    }
    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.0.as_mut_ptr().cast()
    }
    pub fn len(&self) -> usize {
        size_of_val(&self.0)
    }
    pub fn header(&self) -> &bpf_hdr {
        unsafe { (self.0.as_ptr() as *const bpf_hdr).as_ref().unwrap() }
    }
    pub fn frame(&self) -> &[u8] {
        let header = self.header();
        &self.0[header.bh_hdrlen as usize..][..header.bh_caplen as usize]
    }
    // Shift buffer, and return true is there is another packet waiting
    pub fn wind(&mut self) -> bool {
        let header = self.header();
        if header.bh_hdrlen > 0 {
            let len = header.bh_hdrlen as usize + header.bh_caplen as usize;
            let next = (len + BPF_ALIGNMENT as usize - 1) & !(BPF_ALIGNMENT - 1) as usize;
            let remaining = self.len() - next;
            self.0.copy_within(next.., 0);
            self.0[remaining..].iter_mut().for_each(|b| *b = 0);
        }
        self.header().bh_hdrlen > 0
    }
}

pub struct BsdBpfSocket {
    filename: CString,
    interface: CString,
    address: EtherAddr,
    fd: RawFd,
    ethertype: EtherType,
    buffer: BpfBuffer,
}
impl BsdBpfSocket {
    pub(crate) fn new(
        ethertype: EtherType,
        interface: &CStr,
        address: EtherAddr,
    ) -> Result<BsdBpfSocket> {
        let mut bpf_devices: Vec<_> = read_dir("/dev")?
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with("bpf"))
            .map(|e| e.path())
            .collect();

        bpf_devices.sort();

        let mut fd = -1;
        let mut filename = Default::default();
        for path in bpf_devices {
            filename = CString::new(path.into_os_string().into_vec()).unwrap();
            fd = unsafe { open(filename.as_ptr(), O_RDWR, 0) };
            if fd == -1 {
                let err = Error::last_os_error();
                if err.raw_os_error() == Some(EBUSY) {
                    continue;
                }
                return Err(err);
            }
            break;
        }
        if fd == -1 {
            return Err(Error::new(
                ErrorKind::Other,
                "No /dev/bpf devices available",
            ));
        }
        let mut socket = BsdBpfSocket {
            filename,
            interface: interface.to_owned(),
            address,
            fd,
            ethertype,
            buffer: BpfBuffer::new(),
        };

        socket.set_buffer_len(socket.buffer.len() as c_uint)?;
        socket.set_interface(interface)?;
        socket.set_immediate(true)?;

        Ok(socket)
    }
    fn stats(&self) -> Result<bpf_stat> {
        let mut value = bpf_stat::default();
        if unsafe { ioctl(self.fd, BIOCGSTATS, &mut value) } == -1 {
            return Err(Error::last_os_error());
        }
        Ok(value)
    }
    fn set_buffer_len(&mut self, mut len: c_uint) -> Result<()> {
        if unsafe { ioctl(self.fd, BIOCSBLEN, &mut len) } == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
    fn set_interface(&mut self, interface: &CStr) -> Result<()> {
        let mut ifreq = [0u8; 128];
        let ifr_name = &mut ifreq[..IF_NAMESIZE];
        let bytes = interface.to_bytes_with_nul();
        ifr_name[..bytes.len()].copy_from_slice(bytes);
        if unsafe { ioctl(self.fd, BIOCSETIF, &mut ifreq) } == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
    fn set_immediate(&mut self, on: bool) -> Result<()> {
        let mut value: c_uint = if on { 1 } else { 0 };
        if unsafe { ioctl(self.fd, BIOCIMMEDIATE, &mut value) } == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<()> {
        let mut tv = if let Some(timeout) = timeout {
            timeval {
                tv_sec: timeout.as_secs() as time_t,
                tv_usec: timeout.subsec_micros() as suseconds_t,
            }
        } else {
            timeval {
                tv_sec: 0,
                tv_usec: 0,
            }
        };
        if unsafe { ioctl(self.fd, BIOCSRTIMEOUT, &mut tv) } == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}
impl EtherSocket for BsdBpfSocket {
    type Error = Error;
    fn sendto(&mut self, destination: EtherAddr, payload: &[u8]) -> Result<()> {
        let mut buffer = [0u8; 1514];
        let data = &mut buffer;
        let (dest_addr, data) = data.split_at_mut(6);
        let (from_addr, data) = data.split_at_mut(6);
        let (ethertype, data) = data.split_at_mut(2);
        let data = &mut data[..payload.len()];
        dest_addr.copy_from_slice(&destination.as_bytes());
        from_addr.copy_from_slice(&EtherAddr::default().as_bytes());
        ethertype.copy_from_slice(&self.ethertype.as_bytes());
        data.copy_from_slice(payload);
        let mut range = data.as_ptr_range();
        range.start = buffer.as_ptr();

        let buf = range.start.cast();
        let len = unsafe { range.end.offset_from(range.start) as usize };
        let size = unsafe { write(self.fd, buf, len) };
        if size == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
    fn recvfrom<'a>(
        &mut self,
        buffer: &'a mut [u8],
        mut timeout: Option<Duration>,
    ) -> Result<Option<(EtherAddr, &'a [u8])>> {
        if !self.buffer.wind() {
            // If there are no packets remaining in our buffer, then get a new packet
            self.set_read_timeout(timeout)?;
            // Reading from BPF fd returns buffer with a BPF header
            let read_time = Instant::now();
            let size = unsafe { read(self.fd, self.buffer.as_mut_ptr(), self.buffer.len()) };
            if size == -1 {
                let e = Error::last_os_error();
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
            if size == 0 {
                return Ok(None);
            }
            if let Some(previous_timeout) = timeout {
                let new_timeout = previous_timeout.checked_sub(read_time.elapsed());
                timeout = Some(new_timeout.unwrap_or_default());
            }
        }

        // Process ethernet header
        let data = self.buffer.frame();
        let (dest_addr, data) = data.split_at(6);
        let (from_addr, data) = data.split_at(6);
        let (ethertype, data) = data.split_at(2);
        let _ = dest_addr;

        let ethertype = EtherType::from_slice(ethertype);
        if ethertype != self.ethertype {
            // Skip packets that don't match our ethertype
            return self.recvfrom(buffer, timeout);
        }

        let addr = EtherAddr::from_slice(from_addr);
        if addr == self.address {
            // Skip packets that are from us
            return self.recvfrom(buffer, timeout);
        }

        let payload = &mut buffer[0..data.len()];
        payload.copy_from_slice(data);
        Ok(Some((addr, payload)))
    }
}
impl std::fmt::Debug for BsdBpfSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let stats = self.stats().unwrap_or_default();
        write!(
            f,
            "BsdBpfSocket({:?}, interface={:?}, fd={}, ethertype={:?}, received={}, dropped={})",
            self.filename,
            self.interface,
            self.fd,
            self.ethertype,
            stats.packets_received,
            stats.packets_dropped
        )?;
        Ok(())
    }
}
impl Drop for BsdBpfSocket {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}
