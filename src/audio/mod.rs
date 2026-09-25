use agb::{
    include_wav,
    sound::mixer::{Mixer, SoundChannel, SoundData},
};
use agb_tracker::{Track, include_xm};

pub static BGM: Track = include_xm!("sfx/bgm.xm");
pub static BALL_PADDLE_HIT: SoundData = include_wav!("sfx/paddle-hit.wav");
pub static BALL_WALL_HIT: SoundData = include_wav!("sfx/wall-hit.wav");
pub static BALL_WALL_FAIL: SoundData = include_wav!("sfx/fail.wav");

pub enum SfxHit {
    Paddle,
    Wall,
    Fail,
}

pub fn play_hit(mixer: &mut Mixer, tgt: SfxHit) {
    let hit_sound = match tgt {
        SfxHit::Paddle => SoundChannel::new(BALL_PADDLE_HIT),
        SfxHit::Wall => SoundChannel::new(BALL_WALL_HIT),
        SfxHit::Fail => SoundChannel::new(BALL_WALL_FAIL),
    };
    mixer.play_sound(hit_sound);
}
