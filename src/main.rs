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
use agb::display::Priority;
use agb::display::object::Object;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::fixnum::{Num, Rect, Vector2D, num, rect, vec2};
use agb::include_aseprite;
use agb::include_background_gfx;
use agb::input::Button;
use agb::sound::mixer::{Frequency, Mixer, SoundChannel};
use agb::{include_wav, sound::mixer::SoundData};
use agb_tracker::{Track, Tracker, include_xm};
include_background_gfx!(
    mod background,
    PLAY_FIELD => 256 deduplicate "gfx/background.aseprite",
);

include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

static BGM: Track = include_xm!("sfx/bgm.xm");
static BALL_PADDLE_HIT: SoundData = include_wav!("sfx/paddle-hit.wav");
static BALL_WALL_HIT: SoundData = include_wav!("sfx/wall-hit.wav");

enum SfxHit {
    Paddle,
    Wall,
}

fn play_hit(mixer: &mut Mixer, tgt: SfxHit) {
    let hit_sound = match tgt {
        SfxHit::Paddle => SoundChannel::new(BALL_PADDLE_HIT),
        SfxHit::Wall => SoundChannel::new(BALL_WALL_HIT),
    };
    mixer.play_sound(hit_sound);
}

type Fixed = Num<i32, 8>;

const VEL: i32 = 10;

const TURN_ON_BGM: bool = false;
struct Paddle {
    pos: Vector2D<Fixed>,
    dir: i32,
}

impl Paddle {
    fn new(x: i32, y: i32) -> Self {
        Self {
            pos: vec2(x.into(), y.into()),
            dir: 0,
        }
    }

    fn collision_rect(&self) -> Rect<Fixed> {
        rect(self.pos, vec2(num!(16), num!(16 * 3)))
    }

    fn set_y(&mut self, y: Fixed) {
        self.pos.y = y;
        self.move_by(0, false);
    }

    fn move_by(&mut self, y: i32, boost: bool) {
        self.dir = y;

        let boost_val = if boost { VEL * y } else { y };
        self.pos += vec2(num!(0), Fixed::from(y + boost_val));

        self.pos.y = self
            .pos
            .y
            .clamp(num!(0), num!(agb::display::HEIGHT - 16 * 3));
    }

    fn show(&self, frame: &mut agb::display::GraphicsFrame, flip: bool) {
        let sprite_pos = self.pos.round();
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(sprite_pos)
            .set_hflip(flip)
            .show(frame);
        Object::new(sprites::PADDLE_MID.sprite(0))
            .set_pos(sprite_pos + vec2(0, 16))
            .set_hflip(flip)
            .show(frame);
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(sprite_pos + vec2(0, 32))
            .set_vflip(true)
            .set_hflip(flip)
            .show(frame);
    }
}

struct Ball {
    pos: Vector2D<Fixed>,
    vel: Vector2D<Fixed>,
}

impl Ball {
    fn new(pos: Vector2D<Fixed>, vel: Vector2D<Fixed>) -> Self {
        Self { pos, vel }
    }

    fn update(&mut self, a: &Paddle, b: &Paddle) -> Option<SfxHit> {
        let mut sfx_type = None;

        let next_pos = self.pos + self.vel;
        let ball_rect = rect(next_pos, vec2(num!(16), num!(16)));

        let a_rect = a.collision_rect();
        if a_rect.touches(ball_rect) {
            self.vel.x = self.vel.x.abs();
            let y_difference = (ball_rect.centre().y - a_rect.centre().y) / 32;
            self.vel.y += y_difference;

            sfx_type = Some(SfxHit::Paddle)
        }

        let b_rect = b.collision_rect();
        if b_rect.touches(ball_rect) {
            self.vel.x = -self.vel.x.abs();
            let y_difference = (ball_rect.centre().y - b_rect.centre().y) / 32;
            self.vel.y += y_difference;

            sfx_type = Some(SfxHit::Paddle)
        }

        if self.pos.x <= num!(0) || self.pos.x >= num!(agb::display::WIDTH - 16) {
            self.vel.x *= -1;

            sfx_type = Some(SfxHit::Wall)
        };

        if self.pos.y <= num!(0) || self.pos.y >= num!(agb::display::HEIGHT - 16) {
            self.vel.y *= -1;

            sfx_type = Some(SfxHit::Wall)
        };

        self.pos += self.vel;
        self.vel.y = self.vel.y.clamp(num!(-VEL), num!(VEL));

        sfx_type
    }

    fn show(&self, frame: &mut agb::display::GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos.round())
            .show(frame);
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
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

    let mut ball = Ball::new(
        vec2(
            num!(agb::display::WIDTH / 2),
            num!(agb::display::HEIGHT / 2),
        ),
        vec2(num!(2), num!(0.5)),
    );

    let mut paddle_a = Paddle::new(8, 8);
    let mut paddle_b = Paddle::new(240 - 16 - 8, 8);

    let mut button_controller = agb::input::ButtonController::new();

    loop {
        button_controller.update();

        paddle_a.move_by(
            button_controller.y_tri() as i32,
            button_controller.is_pressed(Button::A),
        );

        if let Some(sfxhit) = ball.update(&paddle_a, &paddle_b) {
            play_hit(&mut mixer, sfxhit);
        };

        paddle_b.set_y(ball.pos.y);

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame, false);
        paddle_b.show(&mut frame, true);
        bg.show(&mut frame);

        // ~
        if TURN_ON_BGM {
            tracker.step(&mut mixer);
        }

        mixer.frame();
        frame.commit();
    }
}
