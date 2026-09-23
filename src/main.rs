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

use agb::display::object::Object;
use agb::fixnum::{Vector2D, vec2};
use agb::include_aseprite;
use agb::input::Button;

include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

const VEL: i32 = 10;

struct Paddle {
    pos: Vector2D<i32>,
}

impl Paddle {
    fn new(x: i32, y: i32) -> Self {
        Self { pos: vec2(x, y) }
    }

    fn move_by(&mut self, y: i32, boost: bool) {
        let boost_val = boost as i32 * VEL * y;
        self.pos += vec2(0, y + boost_val);
        self.pos.y = self.pos.y.clamp(0, agb::display::HEIGHT - 16 * 3);
    }

    fn show(&self, frame: &mut agb::display::GraphicsFrame, flip: bool) {
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(self.pos)
            .set_hflip(flip)
            .show(frame);
        Object::new(sprites::PADDLE_MID.sprite(0))
            .set_pos(self.pos + vec2(0, 16))
            .set_hflip(flip)
            .show(frame);
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(self.pos + vec2(0, 32))
            .set_vflip(true)
            .set_hflip(flip)
            .show(frame);
    }
}

struct Ball {
    pos: Vector2D<i32>,
    vel: Vector2D<i32>,
}

impl Ball {
    fn new(pos: Vector2D<i32>, vel: Vector2D<i32>) -> Self {
        Self { pos, vel }
    }

    fn update(&mut self) {
        self.pos += self.vel;

        if self.pos.x == 0 || self.pos.x == (agb::display::WIDTH - 16) {
            self.vel.x *= -1;
        };

        if self.pos.y == 0 || self.pos.y == (agb::display::HEIGHT - 16) {
            self.vel.y *= -1;
        };
    }

    fn show(&self, frame: &mut agb::display::GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos)
            .show(frame);
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut ball = Ball::new(vec2(50, 50), vec2(1, 1));

    let mut paddle_a = Paddle::new(8, 8);
    let mut paddle_b = Paddle::new(240 - 16 - 8, 8);

    let mut button_controller = agb::input::ButtonController::new();

    loop {
        button_controller.update();

        paddle_a.move_by(
            button_controller.y_tri() as i32,
            button_controller.is_pressed(Button::A),
        );

        ball.update();

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame, false);
        paddle_b.show(&mut frame, true);

        frame.commit();
    }
}
