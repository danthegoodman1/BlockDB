use io_uring::{opcode, IoUring};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use tokio::sync::Mutex;

const BLOCK_SIZE: usize = 4096; // Typical block size, adjust as needed

/**
 * IOUringDevice, typically representing a raw block device (NVMe, disk, etc.) but can be a file.
 *
 * Uses io_uring and 4k aligned buffers to read and write data.
 *
 * Works with direct files for testing, in prod you can point at unformatted NVMe devices
 */
pub struct IOUringDevice {
    fd: Option<std::fs::File>,
    ring: Arc<Mutex<IoUring>>,
}

#[repr(align(4096))]
struct AlignedPage([u8; BLOCK_SIZE]);

impl IOUringDevice {
    pub fn new(device_path: &str, ring: Arc<Mutex<IoUring>>) -> std::io::Result<Self> {
        let fd = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(libc::O_DIRECT)
            .open(device_path)?;

        Ok(Self { fd: Some(fd), ring })
    }

    pub async fn read_block(&mut self, offset: u64) -> std::io::Result<AlignedPage> {
        let mut page = AlignedPage([0; BLOCK_SIZE]);
        let fd = io_uring::types::Fd(self.fd.as_ref().unwrap().as_raw_fd());

        let read_e = opcode::Read::new(fd, page.0.as_mut_ptr(), page.0.len() as _)
            .offset(offset)
            .build()
            .user_data(0x42);

        // Lock the ring for this operation
        let mut ring = self.ring.lock().await;

        unsafe {
            ring.submission()
                .push(&read_e)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }

        ring.submit_and_wait(1)?;

        // Process completion
        while let Some(cqe) = ring.completion().next() {
            if cqe.result() < 0 {
                return Err(std::io::Error::from_raw_os_error(-cqe.result()));
            }
        }

        Ok(page)
    }

    pub async fn write_block(&mut self, offset: u64, data: AlignedPage) -> std::io::Result<()> {
        let fd = io_uring::types::Fd(self.fd.as_ref().unwrap().as_raw_fd());

        let write_e = opcode::Write::new(fd, data.0.as_ptr(), data.0.len() as _)
            .offset(offset)
            .build()
            .user_data(0x43);

        // Lock the ring for this operation
        let mut ring = self.ring.lock().await;

        unsafe {
            ring.submission()
                .push(&write_e)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }

        ring.submit_and_wait(1)?;

        // Process completion
        while let Some(cqe) = ring.completion().next() {
            if cqe.result() < 0 {
                return Err(std::io::Error::from_raw_os_error(-cqe.result()));
            }
        }

        Ok(())
    }
}

impl Drop for IOUringDevice {
    fn drop(&mut self) {
        if let Some(fd) = self.fd.take() {
            drop(fd);
        }
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Create a shared io_uring instance
//     let ring = Arc::new(Mutex::new(IoUring::new(128)?));

//     // Create a new device instance
//     let mut device = IOUringDevice::new("/tmp/test1", ring)?;

//     // Example: Write to offset 0
//     let mut write_data = [0u8; BLOCK_SIZE];
//     let hello = b"Hello, world!\n";
//     write_data[..hello.len()].copy_from_slice(hello);
//     let write_page = AlignedPage(write_data);

//     println!("Writing bytes: {:?}", &write_page.0[..hello.len()]);

//     device.write_block(0, write_page).await?;
//     println!("Write completed");

//     // Example: Read from offset 0
//     let data = device.read_block(0).await?;
//     println!("Read {} bytes", data.0.len());

//     println!("Message: {}", String::from_utf8_lossy(&data.0));

//     Ok(())
// }
