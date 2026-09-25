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

use num_traits::Signed;

use agb::display::{
    GraphicsFrame, Priority,
    object::Object,
    tiled::{RegularBackground, RegularBackgroundSize, TileFormat},
};
use agb::fixnum::{Num, Rect, Vector2D, num, rect, vec2};
use agb::input::Button;
use agb::sound::mixer::{Frequency, Mixer, SoundChannel, SoundData};
use agb::{include_aseprite, include_background_gfx, include_wav};
use agb_tracker::{Track, Tracker, include_xm};

include_background_gfx!(
    mod background,
    PLAY_FIELD => 256 deduplicate "gfx/background.aseprite",
    SCORE => deduplicate "gfx/player-health.aseprite",
);

include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite",
    "gfx/cpu-health.aseprite"
);

static BGM: Track = include_xm!("sfx/bgm.xm");
static BALL_PADDLE_HIT: SoundData = include_wav!("sfx/paddle-hit.wav");
static BALL_WALL_HIT: SoundData = include_wav!("sfx/wall-hit.wav");
static BALL_WALL_FAIL: SoundData = include_wav!("sfx/fail.wav");

type Fixed = Num<i32, 8>;

enum SfxHit {
    Paddle,
    Wall,
    Fail,
}

enum PaddleFail {
    A,
    B,
}

fn play_hit(mixer: &mut Mixer, tgt: SfxHit) {
    let hit_sound = match tgt {
        SfxHit::Paddle => SoundChannel::new(BALL_PADDLE_HIT),
        SfxHit::Wall => SoundChannel::new(BALL_WALL_HIT),
        SfxHit::Fail => SoundChannel::new(BALL_WALL_FAIL),
    };
    mixer.play_sound(hit_sound);
}

fn show_cpu_health(paddle: &Paddle, frame: &mut GraphicsFrame) {
    const TEXT_HEART_GAP: i32 = 3;
    let top_left = vec2(agb::display::WIDTH - 4 - (2 + 3) * 8 - TEXT_HEART_GAP, 4);
    Object::new(sprites::CPU.sprite(0))
        .set_pos(top_left)
        .show(frame);
    Object::new(sprites::CPU.sprite(1))
        .set_pos(top_left + vec2(8, 0))
        .show(frame);

    for i in 0..3 {
        let heart_frame = if i < paddle.health { 0 } else { 1 };
        Object::new(sprites::HEART.sprite(heart_frame))
            .set_pos(top_left + vec2(16 + i * 8 + TEXT_HEART_GAP, 0))
            .show(frame);
    }
}

const VEL: i32 = 10;
const INITIAL_HEALTH: i32 = 3;
const TURN_ON_BGM: bool = false;

struct Paddle {
    pos: Vector2D<Fixed>,
    dir: i32,
    health: i32,
}

impl Paddle {
    fn new(x: i32, y: i32) -> Self {
        Self {
            pos: vec2(x.into(), y.into()),
            dir: 0,
            health: INITIAL_HEALTH,
        }
    }

    fn decrement_health(&mut self) {
        self.health -= 1;
    }

    fn collision_rect(&self) -> Rect<Fixed> {
        rect(self.pos, vec2(num!(16), num!(16 * 3)))
    }

    fn set_y(&mut self, y: Fixed) {
        let diff: i32 = (y - self.pos.y).signum().to_raw();
        self.move_by(diff, false);
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

    fn show(&self, frame: &mut GraphicsFrame, flip: bool) {
        let sprite_pos = self.pos.round();
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(sprite_pos)
            .set_hflip(flip)
            .set_priority(Priority::P1)
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

    fn reset(&mut self) {
        self.pos = vec2(
            num!(agb::display::WIDTH / 2),
            num!(agb::display::HEIGHT / 2),
        );
        self.vel = vec2(num!(2), num!(0.5));
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

            sfx_type = Some(SfxHit::Paddle);
        }

        let b_rect = b.collision_rect();
        if b_rect.touches(ball_rect) {
            self.vel.x = -self.vel.x.abs();
            let y_difference = (ball_rect.centre().y - b_rect.centre().y) / 32;
            self.vel.y += y_difference;

            sfx_type = Some(SfxHit::Paddle);
        }

        if self.pos.x <= num!(0) || self.pos.x >= num!(agb::display::WIDTH - 16) {
            self.vel.x *= -1;
            sfx_type = Some(SfxHit::Wall);
        };

        if self.pos.y <= num!(0) || self.pos.y >= num!(agb::display::HEIGHT - 16) {
            self.vel.y *= -1;
            sfx_type = Some(SfxHit::Wall);
        };

        self.pos += self.vel;
        self.vel.y = self.vel.y.clamp(num!(-VEL), num!(VEL));

        sfx_type
    }

    fn check_loss(&mut self) -> Option<PaddleFail> {
        // left paddle miss
        if self.pos.x <= num!(0) {
            return Some(PaddleFail::A);
        }

        // right paddle miss
        if self.pos.x >= num!(agb::display::WIDTH - 16) {
            return Some(PaddleFail::B);
        }

        None
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos.round())
            .set_priority(Priority::P1)
            .show(frame);
    }
}

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

    let mut player_health_background = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for i in 0..4 {
        player_health_background.set_tile(
            (i, 0),
            &background::SCORE.tiles,
            background::SCORE.tile_settings[i as usize],
        );
    }

    for i in 0..3 {
        let tile_index = if i < paddle_a.health { 4 } else { 5 };
        player_health_background.set_tile(
            (i + 4, 0),
            &background::SCORE.tiles,
            background::SCORE.tile_settings[tile_index],
        );
    }

    player_health_background.set_scroll_pos((-4, -4));

    let mut ball = Ball::new(
        vec2(
            num!(agb::display::WIDTH / 2),
            num!(agb::display::HEIGHT / 2),
        ),
        vec2(num!(2), num!(0.5)),
    );

    loop {
        button_controller.update();

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

            for i in 0..3 {
                let tile_index = if i < paddle_a.health { 4 } else { 5 };
                player_health_background.set_tile(
                    (i + 4, 0),
                    &background::SCORE.tiles,
                    background::SCORE.tile_settings[tile_index],
                );
            }
        }

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame, false);
        paddle_b.show(&mut frame, true);
        bg.show(&mut frame);
        player_health_background.show(&mut frame);
        show_cpu_health(&paddle_b, &mut frame);

        // ~
        if TURN_ON_BGM {
            tracker.step(&mut mixer);
        }

        mixer.frame();
        frame.commit();
    }
}
