#[cfg(target_os = "linux")]
mod linux_impl {
    const BLOCK_SIZE: usize = 4096;
    use io_uring::{opcode, IoUring};
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub struct IOUringDevice {
        fd: Option<std::fs::File>,
        ring: Arc<Mutex<IoUring>>,
    }

    #[repr(align(4096))]
    pub struct AlignedPage(pub [u8; BLOCK_SIZE]);

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
}

#[cfg(target_os = "linux")]
pub use linux_impl::IOUringDevice;

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use io_uring::IoUring;
    use linux_impl::{AlignedPage, IOUringDevice};
    use tokio::sync::Mutex;

    use super::*;
    use std::sync::Arc;

    const BLOCK_SIZE: usize = 4096;

    #[tokio::test]
    async fn test_io_uring_read_write() -> Result<(), Box<dyn std::error::Error>> {
        // Create a shared io_uring instance
        let ring = Arc::new(Mutex::new(IoUring::new(128)?));

        // Create a temporary file path
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        // Create a new device instance
        let mut device = IOUringDevice::new(temp_path, ring)?;

        // Test data
        let mut write_data = [0u8; BLOCK_SIZE];
        let hello = b"Hello, world!\n";
        write_data[..hello.len()].copy_from_slice(hello);
        let write_page = AlignedPage(write_data);

        // Write test
        device.write_block(0, write_page).await?;

        // Read test
        let read_page = device.read_block(0).await?;

        // Verify the contents
        assert_eq!(&read_page.0[..hello.len()], hello);

        Ok(())
    }

    #[tokio::test]
    async fn test_io_uring_sqpoll() -> Result<(), Box<dyn std::error::Error>> {
        // Create a shared io_uring instance with SQPOLL enabled
        let ring = Arc::new(Mutex::new(
            IoUring::builder()
                .setup_sqpoll(2000) // 2000ms timeout
                .build(128)?,
        ));

        // Create a temporary file path
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        // Create a new device instance
        let mut device = IOUringDevice::new(temp_path, ring)?;

        // Test data
        let mut write_data = [0u8; BLOCK_SIZE];
        let test_data = b"Testing SQPOLL mode!\n";
        write_data[..test_data.len()].copy_from_slice(test_data);
        let write_page = AlignedPage(write_data);

        // Write test
        device.write_block(0, write_page).await?;

        // Read test
        let read_page = device.read_block(0).await?;

        // Verify the contents
        assert_eq!(&read_page.0[..test_data.len()], test_data);

        Ok(())
    }
}
