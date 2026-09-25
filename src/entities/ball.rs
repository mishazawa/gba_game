use agb::{
    display::{GraphicsFrame, Priority, object::Object},
    fixnum::{Vector2D, num, rect, vec2},
};

use crate::{
    audio::SfxHit,
    data::{Fixed, VEL, sprites},
    entities::{Paddle, PaddleFail},
};

pub struct Ball {
    pub pos: Vector2D<Fixed>,
    vel: Vector2D<Fixed>,
}

impl Ball {
    pub fn new(pos: Vector2D<Fixed>, vel: Vector2D<Fixed>) -> Self {
        Self { pos, vel }
    }

    pub fn reset(&mut self) {
        self.pos = vec2(
            num!(agb::display::WIDTH / 2),
            num!(agb::display::HEIGHT / 2),
        );
        self.vel = vec2(num!(2), num!(0.5));
    }

    pub fn update(&mut self, a: &Paddle, b: &Paddle) -> Option<SfxHit> {
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

    pub fn check_loss(&mut self) -> Option<PaddleFail> {
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

    pub fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos.round())
            .set_priority(Priority::P1)
            .show(frame);
    }
}
