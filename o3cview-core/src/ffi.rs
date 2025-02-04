use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH, Viewer};

#[unsafe(no_mangle)]
pub extern "C" fn o3cview_init() -> *mut Viewer {
    if let Ok(viewer) = Viewer::new() {
        Box::into_raw(Box::new(viewer))
    } else {
        std::ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn o3cview_free(viewer: *mut Viewer) {
    unsafe {
        std::mem::drop(Box::from_raw(viewer));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn o3cview_get_frame(viewer: *mut Viewer, fb: *mut u8) {
    unsafe {
        let viewer = &mut *viewer;
        viewer.get_frame(
            std::slice::from_raw_parts_mut(fb, DISPLAY_WIDTH * DISPLAY_HEIGHT * 2)
                .try_into()
                .unwrap(),
        );
    }
}
