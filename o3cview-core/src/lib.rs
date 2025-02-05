mod ffi;

use hidapi::{HidApi, HidDevice, HidError};

const SAYO_VENDOR_ID: u16 = 0x8089;
const API_V2_FAST_USAGE_PAGE: u16 = 0xFF12;
const API_V2_FAST_REPORT_ID: u8 = 0x22;

pub const DISPLAY_WIDTH: usize = 160;
pub const DISPLAY_HEIGHT: usize = 80;

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
        ret.req[0] = API_V2_FAST_REPORT_ID;
        ret.req[1] = 0x03;
        ret.req[4] = 0x08;
        ret.req[6] = 0x25;

        Ok(ret)
    }

    fn open_device(&mut self) -> bool {
        if self.hid.reset_devices().is_err() || self.hid.add_devices(SAYO_VENDOR_ID, 0).is_err() {
            return false;
        }
        for x in self.hid.device_list() {
            if x.vendor_id() == SAYO_VENDOR_ID && x.usage_page() == API_V2_FAST_USAGE_PAGE {
                if let Ok(device) = x.open_device(&self.hid) {
                    if device.set_blocking_mode(false).is_err() {
                        return false;
                    }
                    self.device = Some(device);
                    return true;
                }
            }
        }

        false
    }

    fn try_read_frame(
        &mut self,
        fb: &mut [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 2],
    ) -> Result<(), HidError> {
        let device = self.device.as_mut().unwrap();

        const CHUNK_SIZE: usize = 1024 - 0xC;
        for i in (0..fb.len()).step_by(CHUNK_SIZE) {
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
        'outer: for (i, x) in fb.chunks_mut(CHUNK_SIZE).enumerate() {
            for _ in 0..5 {
                if device.read_timeout(&mut self.res, 4)? == 0 {
                    // Probably a dropped packet somewhere
                    return Ok(());
                }

                if self.res[0] == API_V2_FAST_REPORT_ID && u32::from_le_bytes(self.res[8..12].try_into().unwrap()) as usize == i * CHUNK_SIZE {
                    // Copy to framebuffer
                    x.copy_from_slice(&self.res[12..(12 + x.len())]);
                    continue 'outer;
                }
            }

            return Err(HidError::HidApiErrorEmpty);
        }

        Ok(())
    }

    pub fn get_frame(&mut self, fb: &mut [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 2]) {
        if self.device.is_none() && !self.open_device() {
            fb.clone_from(include_bytes!("../nodevice.bin"));
            return;
        }

        if self.try_read_frame(fb).is_err() {
            self.device = None;
        }
    }
}
