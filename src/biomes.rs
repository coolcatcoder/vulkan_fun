pub fn get_biome(noise_x: f64, noise_y: f64) -> usize {
    // To handle overlapping shapes we should instead go through each biome, if it is a correct biome, add it to a vector, then pick randomly from the vector at the end.
    for biome_index in 0..BIOMES.len() {
        if BIOMES[biome_index].aabb.point_intersects(noise_x, noise_y) {
            return biome_index;
        }
    }

    0
}

pub const SPRITE_SIZE: (f32, f32) = (1.0 / 42.0, 1.0);

pub const BIOMES: [Biome; 1] = [Biome {
    aabb: Aabb {
        size_x: 1.0,
        size_y: 1.0,
        position_x: 0.0,
        position_y: 0.0,
    },
    random_pattern: PatternArray {
        starting_index: 0,
        length: 1,
    },
    simplex_pattern: PatternArray {
        starting_index: 0,
        length: 3,
    },
    simplex_smoothed_pattern: PatternArray {
        starting_index: 0,
        length: 0,
    },
}];

pub const RANDOM_PATTERN_MAP_OBJECTS: [RandomPatternMapObject; 1] = [RandomPatternMapObject {
    chance: 10,
    priority: 1,
    behaviour: CollisionBehaviour::None,
    rendering_size: (0.5, 0.5),
    collision_size: (0.5, 0.5),
    uv: (5.0 * SPRITE_SIZE.0, 0.0),
}];

pub const SIMPLEX_PATTERN_MAP_OBJECTS: [SimplexPatternMapObject; 3] = [
    SimplexPatternMapObject {
        // circus rock
        chance: 100,
        priority: 2,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 1,
        acceptable_noise: (0.0, 1.0),
        noise_scale: 0.2,
        uv: (7.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // flower test
        chance: 75,
        priority: 1,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (0.0, 0.0),
        seed: 2,
        acceptable_noise: (0.2, 0.5),
        noise_scale: 0.1,
        uv: (16.0 * SPRITE_SIZE.0, 0.0),
    },
    SimplexPatternMapObject {
        // Large test
        chance: 100,
        priority: 3,
        behaviour: CollisionBehaviour::None,
        rendering_size: (1.0, 1.0),
        collision_size: (1.0, 1.0),
        seed: 3,
        acceptable_noise: (-1.0, -0.6),
        noise_scale: 0.05,
        uv: (9.0 * SPRITE_SIZE.0, 0.0),
    },
];

pub const SIMPLEX_SMOOTHED_PATTERN_MAP_OBJECTS: [SimplexSmoothedPatternMapObject; 0] = [];

#[derive(Debug)]
pub struct RandomPatternMapObject {
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub uv: (f32, f32),
}

#[derive(Debug)]
pub struct SimplexPatternMapObject {
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub rendering_size: (f32, f32),
    pub collision_size: (f32, f32),
    pub seed: u8,
    pub acceptable_noise: (f64, f64),
    pub noise_scale: f64,
    pub uv: (f32, f32),
}

#[derive(Debug)]
pub struct SimplexSmoothedPatternMapObject {
    pub chance: u8,
    pub priority: u8,
    pub behaviour: CollisionBehaviour,
    pub size: f32, // Bad name? This is the size of a single square during marching squares.
    pub seed: u8,
    pub acceptable_noise: (f64, f64),
    pub noise_scale: f64,
}

#[derive(Copy, Clone)]
pub enum MapObject {
    None,
    RandomPattern(u8),
    SimplexPattern(u8),
    SimplexSmoothedPattern(u8),
}

#[derive(Debug)]
pub struct Biome {
    aabb: Aabb,
    pub random_pattern: PatternArray,
    pub simplex_pattern: PatternArray,
    pub simplex_smoothed_pattern: PatternArray,
}

#[derive(Debug)]
pub struct Aabb {
    size_x: f64,
    size_y: f64,

    position_x: f64,
    position_y: f64,
}

impl Aabb {
    fn point_intersects(&self, position_x: f64, position_y: f64) -> bool {
        position_x < self.position_x + self.size_x
            && position_x > self.position_x
            && position_y < self.position_y + self.size_y
            && position_y > self.position_y
    }
}

#[derive(Debug)]
pub struct PatternArray {
    pub starting_index: u8,
    pub length: u8,
}

#[derive(Debug)]
pub enum CollisionBehaviour {
    None,
    Consume(u8),
}
