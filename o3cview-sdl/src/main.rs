#![windows_subsystem = "windows"]

use std::time::Duration;

use o3cview_core::{DISPLAY_HEIGHT, DISPLAY_WIDTH, Viewer};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    pixels::{PixelFormat, PixelMasks},
    sys::{render::SDL_SetTextureScaleMode, surface::SDL_SCALEMODE_NEAREST},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl = sdl3::init()?;
    let sdl_video = sdl.video()?;

    let window = sdl_video
        .window("o3cview", DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
        .build()?;
    let mut canvas = window.into_canvas();

    let tex_creator = canvas.texture_creator();
    let mask = PixelMasks {
        bpp: 16,
        rmask: 0xF800,
        gmask: 0x07E0,
        bmask: 0x001F,
        amask: 0x0000,
    };
    let mut tex = tex_creator.create_texture_streaming(
        PixelFormat::from_masks(mask),
        DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
    )?;
    unsafe {
        SDL_SetTextureScaleMode(tex.raw(), SDL_SCALEMODE_NEAREST);
    }

    let mut viewer = Viewer::new()?;
    let mut fb = [0u8; DISPLAY_WIDTH * DISPLAY_HEIGHT * 2];

    let mut event_pump = sdl.event_pump()?;
    let mut scale = 1;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Comma),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Period),
                    ..
                } => {
                    if matches!(
                        event,
                        Event::KeyDown {
                            keycode: Some(Keycode::Comma),
                            ..
                        }
                    ) {
                        if scale > 1 {
                            scale -= 1;
                        }
                    } else {
                        scale += 1;
                    }
                    canvas
                        .window_mut()
                        .set_size(scale * DISPLAY_WIDTH as u32, scale * DISPLAY_HEIGHT as u32)?;
                }
                _ => {}
            }
        }

        viewer.get_frame(&mut fb);
        tex.update(None, &fb, DISPLAY_WIDTH * 2)?;
        canvas.copy(&tex, None, None)?;

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
