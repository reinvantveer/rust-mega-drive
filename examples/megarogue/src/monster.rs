#[derive(Copy, Clone)]
pub struct Monster {
    pub position:       (i16, i16),
    pub map_symbol:     &'static str,
    pub hit_message:    &'static str,
    pub miss_message:   &'static str,
}

impl Monster {
    pub fn player1() -> Self {
        Self {
            // Set default position
            position: (0, 0),
            map_symbol: "@",
            hit_message: "You hit!",
            miss_message: "You miss"
        }
    }

    pub fn kobold() -> Self {
        Self {
            // Again, default position
            position: (0, 0),
            map_symbol: "k",
            hit_message: "The kobold hits!",
            miss_message: "The kobold misses"
        }
    }
}