/// This module contains the implementation for the RTT protocol. It's not meant to be used directly
/// in user code, and therefore mostly undocumented. The module is only public so that it can be
/// accessed from the rtt_init! macro.
use core::cmp::min;
use core::ptr;
use vcell::VolatileCell;

// Note: this is zero-initialized in the initialization macro so all zeros must be a valid value
#[repr(C)]
pub struct RttHeader {
    id: [u8; 16],
    max_up_channels: VolatileCell<usize>,
    max_down_channels: VolatileCell<usize>,
    // Followed in memory by:
    // up_channels: [Channel; max_up_channels]
    // down_channels: [Channel; down_up_channels]
}

impl RttHeader {
    pub unsafe fn init(&mut self, max_up_channels: usize, max_down_channels: usize) {
        // Copy the ID in two parts to avoid having the ID string in memory in full

        ptr::copy_nonoverlapping(b"SEGGER R_" as *const u8, self.id.as_mut_ptr(), 9);

        ptr::copy_nonoverlapping(
            b"TT\0\0\0\0\0\0" as *const u8,
            self.id.as_mut_ptr().offset(8),
            8,
        );

        self.max_up_channels.set(max_up_channels);
        self.max_down_channels.set(max_down_channels);
    }
}

// Note: this is zero-initialized in the initialization macro so all zeros must be a valid value
#[repr(C)]
pub struct RttChannel {
    name: *const u8,
    buffer: *mut u8,
    size: usize,
    write: VolatileCell<usize>,
    read: VolatileCell<usize>,
    flags: VolatileCell<usize>,
}

impl RttChannel {
    pub unsafe fn init(&mut self, name: *const u8, buffer: *mut [u8]) {
        self.name = name;
        self.buffer = buffer as *mut u8;
        self.size = (&*buffer).len();
    }

    // This method should only be called for down channels.
    pub(crate) fn read(&self, mut buf: &mut [u8]) -> usize {
        let (write, mut read) = self.read_pointers();

        let mut total = 0;

        // Read while buffer contains data and output buffer has space (maximum of two iterations)
        while buf.len() > 0 {
            let count = min(self.readable_contiguous(write, read), buf.len());
            if count == 0 {
                break;
            }

            unsafe {
                ptr::copy_nonoverlapping(
                    self.buffer.offset(read as isize),
                    buf.as_mut_ptr(),
                    count,
                );
            }

            total += count;
            read += count;

            if read >= self.size {
                // Wrap around to start
                read = 0;
            }

            buf = &mut buf[count..];
        }

        self.read.set(read);

        total
    }

    // This method should only be called for up channels.
    pub(crate) fn write(&self, mut buf: &[u8]) -> usize {
        let (mut write, read) = self.read_pointers();

        let mut total = 0;

        // Write while buffer has space for data and output contains data (maximum of two iterations)
        while buf.len() > 0 {
            let count = min(self.writable_contiguous(write, read), buf.len());
            if count == 0 {
                break;
            }

            unsafe {
                ptr::copy_nonoverlapping(buf.as_ptr(), self.buffer.offset(write as isize), count);
            }

            total += count;
            write += count;

            if write >= self.size {
                // Wrap around to start
                write = 0;
            }

            buf = &buf[count..];
        }

        self.write.set(write);

        total
    }

    /// Gets the amount of contiguous data available for reading
    fn readable_contiguous(&self, write: usize, read: usize) -> usize {
        (if read > write {
            self.size - read
        } else {
            write - read
        }) as usize
    }

    /// Gets the amount of contiguous space available for writing
    fn writable_contiguous(&self, write: usize, read: usize) -> usize {
        (if read > write {
            read - write - 1
        } else if read == 0 {
            self.size - write - 1
        } else {
            self.size - write
        }) as usize
    }

    /// Gets the total amount of writable space left in the buffer
    /*pub(crate) fn writable(&self) -> usize {
        let (write, read) = self.read_pointers();

        self.writable_contiguous(write, read) + if read < write && read > 0 { read } else { 0 }
    }*/

    fn read_pointers(&self) -> (usize, usize) {
        let write = self.write.get();
        let read = self.read.get();

        if write >= self.size || read >= self.size {
            // Pointers have been corrupted. This doesn't happen in well-behaved programs, so
            // attempt to reset the buffer.

            self.write.set(0);
            self.read.set(0);
            return (0, 0);
        }

        (write, read)
    }
}
