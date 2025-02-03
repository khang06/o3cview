use hidapi::{HidApi, HidDevice, HidError};

const SAYO_VENDOR_ID: u16 = 0x8089;
const API_V2_FAST_USAGE_PAGE: u16 = 0xFF12;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 80;

pub struct Viewer {
    hid: HidApi,
    device: Option<HidDevice>,

    // Try to avoid unnecessary allocations
    req: [u8; 1024],
    res: [u8; 1024],
}

impl Viewer {
    pub fn new() -> Result<Self, HidError> {
        let mut ret = Self {
            hid: HidApi::new()?,
            device: None,

            req: [0; 1024],
            res: [0; 1024],
        };
        ret.open_device();

        // TODO: Document what this stuff means
        ret.req[0] = 0x22;
        ret.req[1] = 0x03;
        ret.req[4] = 0x08;
        ret.req[6] = 0x25;

        Ok(ret)
    }

    fn open_device(&mut self) -> bool {
        for x in self.hid.device_list() {
            if x.vendor_id() == SAYO_VENDOR_ID && x.usage_page() == API_V2_FAST_USAGE_PAGE {
                if let Ok(device) = x.open_device(&self.hid) {
                    self.device = Some(device);
                    return true;
                }
            }
        }

        false
    }

    fn try_read_frame(&mut self, fb: &mut [u8; WIDTH * HEIGHT * 2]) -> Result<(), HidError> {
        let device = self.device.as_mut().unwrap();

        for i in (0..fb.len()).step_by(1024 - 0xC) {
            // Clear checksum
            self.req[2] = 0;
            self.req[3] = 0;

            // Offset
            self.req[8..12].copy_from_slice(&(i as u32).to_le_bytes());

            // Perform checksum
            let mut sum: u16 = 0;
            for y in self.req[..12].chunks_exact(2) {
                sum = sum.wrapping_add(u16::from_le_bytes(y.try_into().unwrap()));
            }
            self.req[2..4].copy_from_slice(&sum.to_le_bytes());

            // Send request
            device.write(&self.req)?;
        }

        // Get response
        'outer: for x in fb.chunks_mut(1024 - 0xC) {
            for _ in 0..5 {
                device.read_timeout(&mut self.res, 250)?;

                if self.res[0] == 0x22 {
                    // Copy to framebuffer
                    x.copy_from_slice(&self.res[12..(12 + x.len())]);
                    continue 'outer;
                }
            }

            return Err(HidError::HidApiErrorEmpty);
        }

        Ok(())
    }

    pub fn get_frame(&mut self, fb: &mut [u8; WIDTH * HEIGHT * 2]) {
        if self.device.is_none() && !self.open_device() {
            fb.clone_from(include_bytes!("../nodevice.bin"));
            return;
        }

        if self.try_read_frame(fb).is_err() {
            self.device = None;
        }
    }
}
