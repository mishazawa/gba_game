use agb::{fixnum::Num, include_aseprite, include_background_gfx};

pub type Fixed = Num<i32, 8>;

pub const VEL: i32 = 10;
pub const INITIAL_HEALTH: i32 = 1;
pub const TURN_ON_BGM: bool = false;

include_background_gfx!(
    pub mod background,
    PLAY_FIELD => 256 deduplicate "gfx/background.aseprite",
    SCORE => deduplicate "gfx/player-health.aseprite",
);

include_aseprite!(
    pub mod sprites,
    "gfx/sprites.aseprite",
    "gfx/cpu-health.aseprite"
);
