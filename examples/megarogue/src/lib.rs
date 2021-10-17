#![no_std]

mod monster;

use core::ptr::{read_volatile, write_volatile};

use megadrive_graphics::Renderer;
use megadrive_input::{Controllers, Button};
use megadrive_util::rng::PseudoRng;
use megadrive_sys::vdp::VDP;
use megadrive_graphics::default_ascii::DEFAULT_FONT_1X1;

use crate::monster::Monster;

static mut NEW_FRAME: u16 = 0;
const BUTTON_THROTTLE_FRAMES: u8 = 8;

#[no_mangle]
pub fn main() -> ! {
    let mut renderer = Renderer::new();
    let mut controllers = Controllers::new();
    let mut vdp = VDP::new();

    let mut rng = PseudoRng::from_seed(42);

    let resolution = vdp.resolution();
    let half_screen_width = resolution.0 >> 1;
    let half_screen_height = resolution.1 >> 1;

    let x_off: i16 = 104 + half_screen_width as i16;
    let y_off: i16 = 128 + half_screen_height as i16;

    vdp.enable_interrupts(false, true, false);
    vdp.enable_display(true);

    // Load the font tiles
    DEFAULT_FONT_1X1.load(&mut vdp);

    let mut button_throttle_countdown = BUTTON_THROTTLE_FRAMES;

    let mut player1 = Monster::player1();
    player1.position = (x_off, y_off);
    let mut monsters: [Option<Monster>; 10] = [None; 10];

    loop {
        renderer.clear();
        controllers.update();

        if button_throttle_countdown > 0 { button_throttle_countdown -= 1; }
        let mut move_direction: (i16, i16) = (0, 0);
        let mut activate_turn = false;

        // Always roll: it updates the seed so that the outcome will be different every game
        let random_number = rng.random();

        // Do not set a new direction if the button throttle is engaged
        if let Some(c) = controllers.controller_state(0) {
            if button_throttle_countdown == 0 {
                if c.down(Button::Up)    { move_direction = (0, -8); move_player1 = true; }
                if c.down(Button::Left)  { move_direction = (-8, 0); move_player1 = true; }
                if c.down(Button::Down)  { move_direction = (0,  8); move_player1 = true; }
                if c.down(Button::Right) { move_direction = (8,  0); move_player1 = true; }
            }

            // Engage the button throttle
            if move_player1 { button_throttle_countdown = BUTTON_THROTTLE_FRAMES; }
        }

        x_off += move_direction.0 as i16;
        y_off += move_direction.1 as i16;

        let player1 = "@";
        DEFAULT_FONT_1X1.blit_text(&mut renderer, player1, x_off as u16, y_off as u16);


        renderer.render(&mut vdp);
        // vsync
        wait_for_vblank();
    }
}

extern "C" { fn wait_for_interrupt(); }

fn wait_for_vblank() {
    unsafe {
        while read_volatile(&NEW_FRAME) == 0 {
            wait_for_interrupt();
        }
        NEW_FRAME = 0;
    }
}

#[no_mangle]
fn vblank() {
    unsafe { write_volatile(&mut NEW_FRAME, 1) };
}
