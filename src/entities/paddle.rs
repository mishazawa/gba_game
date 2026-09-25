use agb::{
    display::{GraphicsFrame, Priority, object::Object},
    fixnum::{Rect, Vector2D, num, rect, vec2},
};
use num_traits::Signed;

use crate::data::{Fixed, INITIAL_HEALTH, VEL, sprites};

pub struct Paddle {
    pos: Vector2D<Fixed>,
    dir: i32,
    pub health: i32,
}

pub enum PaddleFail {
    A,
    B,
}

impl Paddle {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            pos: vec2(x.into(), y.into()),
            dir: 0,
            health: INITIAL_HEALTH,
        }
    }

    pub fn decrement_health(&mut self) {
        self.health -= 1;
    }

    pub fn collision_rect(&self) -> Rect<Fixed> {
        rect(self.pos, vec2(num!(16), num!(16 * 3)))
    }

    pub fn set_y(&mut self, y: Fixed) {
        let diff: i32 = (y - self.pos.y).signum().to_raw();
        self.move_by(diff, false);
    }

    pub fn move_by(&mut self, y: i32, boost: bool) {
        self.dir = y;

        let boost_val = if boost { VEL * y } else { y };
        self.pos += vec2(num!(0), Fixed::from(y + boost_val));

        self.pos.y = self
            .pos
            .y
            .clamp(num!(0), num!(agb::display::HEIGHT - 16 * 3));
    }

    pub fn show(&self, frame: &mut GraphicsFrame, flip: bool) {
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

    pub fn show_health(&self, frame: &mut GraphicsFrame, top_left: Vector2D<i32>) {
        for i in 0..3 {
            let heart_frame = if i < self.health { 0 } else { 1 };
            Object::new(sprites::HEART.sprite(heart_frame))
                .set_pos(top_left + vec2(i * 8, 0))
                .show(frame);
        }
    }
}
