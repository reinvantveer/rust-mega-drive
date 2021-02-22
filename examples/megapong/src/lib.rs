#![no_std]
#![feature(array_chunks)]

use core::panic::PanicInfo;
use megadrive_sys::vdp::{VDP, Sprite, SpriteSize};
use megadrive_input::Controllers;
use core::ptr::{read_volatile, write_volatile};

static mut NEW_FRAME: u16 = 0;

extern "C" {
    fn wait_for_interrupt();
}

#[no_mangle]
pub fn main() -> ! {
    loop {
        let vdp = VDP::new();
        let mut controllers = Controllers::new();

        static TILE_DATA: [u8; 256] = [
            // H
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x11, 0x11, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // E
            0x01, 0x11, 0x11, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x11, 0x11, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x11, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // L
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x11, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // O
            0x01, 0x11, 0x11, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x11, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // W
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x01, 0x01, 0x00,
            0x01, 0x10, 0x11, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // R
            0x01, 0x10, 0x00, 0x00,
            0x01, 0x01, 0x10, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x10, 0x00,
            0x01, 0x11, 0x00, 0x00,
            0x01, 0x01, 0x00, 0x00,
            0x01, 0x00, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // D
            0x01, 0x10, 0x00, 0x00,
            0x01, 0x01, 0x10, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x01, 0x10, 0x00,
            0x01, 0x10, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            // !
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        vdp.set_tiles(1, TILE_DATA.array_chunks());

        let mut frame = 0u16;
        loop {
            controllers.update();
            let c1 = controllers.controller_state(0);
            let buttons = c1.map_or(0, |c| c.get_down_raw());

            // Write sprites
            let mut x = 200;
            let y = 200;

            static TILE_INDICES: [u16; 13] = [1, 2, 3, 3, 4, 0, 5, 4, 6, 3, 7, 8, 9];
            static NEXT: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0];

            let anim_frame = (frame >> 1) & 0x3f;

            for (idx, (i, next)) in TILE_INDICES.iter().cloned().zip(NEXT.iter().cloned()).enumerate() {
                let my_frame = (anim_frame + (idx as u16)) & 0x3f;
                let mut my_y = y + if my_frame >= 32 {
                    63 - my_frame
                } else
                {
                    my_frame
                };

                let down = ((buttons >> idx) & 1) != 0;
                if down {
                    my_y += 100;
                }

                let mut sprite = Sprite::for_tile(i, SpriteSize::Size1x1);
                sprite.link = next;
                sprite.y = my_y;
                sprite.x = x;
                vdp.set_sprites(idx, [sprite].iter());
                x += 7;
            }

            frame = (frame + 1) & 0x7fff;

            // vsync
            unsafe {
                while read_volatile(&NEW_FRAME) == 0 {
                    wait_for_interrupt();
                }
                NEW_FRAME = 0;
            }
        }
    }
}

#[no_mangle]
fn vblank() {
    unsafe { write_volatile(&mut NEW_FRAME, 1) };
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
