// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't have an operating
// system, so most of the content of the standard library doesn't apply.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

// By default no_std crates don't get alloc, so you won't be able to use things like Vec
// until you declare the extern crate. `agb` provides an allocator so it will all work
extern crate alloc;

mod audio;
mod data;
mod entities;

use agb::display::{
    Priority,
    tiled::{RegularBackground, RegularBackgroundSize, TileFormat},
};
use agb::fixnum::{num, vec2};
use agb::input::Button;
use agb::sound::mixer::Frequency;
use agb_tracker::Tracker;

use crate::{
    audio::{BGM, SfxHit, play_hit},
    data::{TURN_ON_BGM, background},
    entities::{Ball, Paddle, PaddleFail},
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut button_controller = agb::input::ButtonController::new();

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    let mut tracker = Tracker::new(&BGM);

    let mut gfx = gba.graphics.get();

    gfx.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::EightBpp,
    );

    bg.fill_with(&background::PLAY_FIELD);

    let mut paddle_a = Paddle::new(8, 8);
    let mut paddle_b = Paddle::new(240 - 16 - 8, 8);

    let mut ball = Ball::new(
        vec2(
            num!(agb::display::WIDTH / 2),
            num!(agb::display::HEIGHT / 2),
        ),
        vec2(num!(2), num!(0.5)),
    );

    let top_left_a = vec2(4, 4);
    let top_left_b = vec2(agb::display::WIDTH - 3 * 8 - 3, 4);

    loop {
        button_controller.update();
        let mut frame = gfx.frame();

        paddle_a.move_by(
            button_controller.y_tri() as i32,
            button_controller.is_pressed(Button::A),
        );

        paddle_b.set_y(ball.pos.y);

        if let Some(sfxhit) = ball.update(&paddle_a, &paddle_b) {
            play_hit(&mut mixer, sfxhit);
        };

        if let Some(p) = ball.check_loss() {
            match p {
                PaddleFail::A => paddle_a.decrement_health(),
                PaddleFail::B => paddle_b.decrement_health(),
            }
            play_hit(&mut mixer, SfxHit::Fail);
            ball.reset();
        }

        ball.show(&mut frame);
        paddle_a.show(&mut frame, false);
        paddle_b.show(&mut frame, true);
        bg.show(&mut frame);

        paddle_a.show_health(&mut frame, top_left_a);
        paddle_b.show_health(&mut frame, top_left_b);

        // ~

        if TURN_ON_BGM {
            tracker.step(&mut mixer);
        }

        mixer.frame();
        frame.commit();
    }
}
