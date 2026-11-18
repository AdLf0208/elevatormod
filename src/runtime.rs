// ElevatorMod for Horizon — the travel planner (pure).
//
// Ported from ElevatorMod 1.21.1-1.11.4 (VsnGamer/ElevatorMod, MIT) for
// Minecraft 1.21.1 NeoForge. Upstream source read for behaviour only; this is a
// clean-room Rust rewrite.
//
// Slice 1: jump up / sneak down between elevator blocks in the same column.
// The origin is the first elevator at most ACTIVATION_RANGE blocks below the
// feet (aborting if its above-cell suffocates); the target is the nearest
// elevator in the travel direction within RANGE (default 384) whose above-cell
// does not suffocate. Jump/sneak detection and the actual teleport live in the
// glue (the pin's native player teleport helper, called from `server_tick`).

/// Every elevator block starts with this (16 dye suffixes).
pub const PREFIX: &str = "elevatorid:elevator_";

/// Upstream defaults (`Config.java:25-60`).
pub const RANGE: i32 = 384;
pub const ACTIVATION_RANGE: i32 = 6;
pub const SAME_COLOR_DEFAULT: bool = false;

/// Whether a block name is one of the 16 elevators.
pub fn is_elevator(name: &str) -> bool {
    name.strip_prefix(PREFIX)
        .is_some_and(|color| COLORS.contains(&color))
}

/// The dye suffix of an elevator block, if it is one.
pub fn color_of(name: &str) -> Option<&str> {
    let color = name.strip_prefix(PREFIX)?;
    COLORS.contains(&color).then_some(color)
}

/// The 16 dye names, in DyeColor order (`Registry.java:31-34`).
pub const COLORS: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

// The landing position helper: the deferred `/tp` command formatted these
// coordinates inline; the native teleport takes the same triple as a value.

/// The feet position a travel lands on: x/z centred on the block (upstream's
/// `precisionTarget`), y at the block's top. The caller keeps the look angles.
pub fn travel_position(block_x: i32, block_z: i32, feet_y: f64) -> [f64; 3] {
    [f64::from(block_x) + 0.5, feet_y, f64::from(block_z) + 0.5]
}

/// One column of blocks around the player (the glue binds x/z).
pub trait World {
    /// The block name at height `y` in the player's column; None when not
    /// observable (unloaded).
    fn block(&mut self, y: i32) -> Option<String>;
    /// Whether the cell at `y` suffocates a standing entity. The glue
    /// approximates `isSuffocating` with a full-collision block.
    fn suffocating(&mut self, y: i32) -> bool;
}

/// Where a jump/sneak would land.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    /// The elevator block's y.
    pub elevator_y: i32,
    /// The feet position after the teleport (the block's top).
    pub feet_y: f64,
}

/// Plan one travel from `feet_y` (the player's feet position) in the column.
///
/// `up` is true for a jump (target above) and false for a sneak (below).
/// `same_color` requires the target to share the origin's dye.
pub fn plan(world: &mut impl World, feet_y: f64, up: bool, same_color: bool) -> Option<Target> {
    let feet_block = feet_y.floor() as i32;
    // Origin: the first elevator at most ACTIVATION_RANGE cells below the feet
    // block. A suffocating cell above it aborts the whole attempt.
    let mut origin = None;
    for offset in 0..ACTIVATION_RANGE {
        let y = feet_block - offset;
        let name = world.block(y)?;
        if is_elevator(&name) {
            if world.suffocating(y + 1) {
                return None;
            }
            origin = Some((y, color_of(&name).map(str::to_owned)));
            break;
        }
    }
    let (origin_y, origin_color) = origin?;
    let step = if up { 1 } else { -1 };
    for distance in 1..=RANGE {
        let y = origin_y + step * distance;
        let Some(name) = world.block(y) else {
            break;
        };
        if !is_elevator(&name) {
            continue;
        }
        if same_color && color_of(&name) != origin_color.as_deref() {
            continue;
        }
        if world.suffocating(y + 1) {
            continue;
        }
        return Some(Target {
            elevator_y: y,
            feet_y: f64::from(y) + 1.0,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    struct Column {
        blocks: HashMap<i32, String>,
        solid: HashSet<i32>,
    }

    impl Column {
        fn new() -> Self {
            Column {
                blocks: HashMap::new(),
                solid: HashSet::new(),
            }
        }
        fn put(&mut self, y: i32, name: &str) {
            self.blocks.insert(y, name.to_owned());
        }
    }

    impl World for Column {
        fn block(&mut self, y: i32) -> Option<String> {
            Some(
                self.blocks
                    .get(&y)
                    .cloned()
                    .unwrap_or_else(|| "minecraft:air".into()),
            )
        }
        fn suffocating(&mut self, y: i32) -> bool {
            self.solid.contains(&y)
        }
    }

    fn elevator(color: &str) -> String {
        format!("{PREFIX}{color}")
    }

    #[test]
    fn the_sixteen_dye_elevators_are_recognised() {
        assert_eq!(color_of(&elevator("white")), Some("white"));
        assert_eq!(color_of(&elevator("light_blue")), Some("light_blue"));
        assert!(is_elevator(&elevator("black")));
        assert_eq!(color_of("elevatorid:elevator_teal"), None);
        assert_eq!(color_of("minecraft:stone"), None);
        assert!(!is_elevator("elevatorid:elevator"));
    }

    #[test]
    fn the_origin_is_the_first_elevator_below_the_feet() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.solid.insert(64); // the floor is not relevant; only above matters
        let target = plan(&mut world, 64.0, true, false);
        assert!(target.is_none(), "no target above");
        let mut world = Column::new();
        world.put(57, &elevator("white")); // feet block 63, offset 6? below scan
        assert!(
            plan(&mut world, 63.0, true, false).is_none(),
            "out of activation range"
        );
        let mut world = Column::new();
        world.put(57, &elevator("white")); // feet 63, offset 6 -> outside 0..6
        world.put(66, &elevator("white"));
        assert!(plan(&mut world, 63.0, true, false).is_none());
    }

    #[test]
    fn a_suffocating_cell_above_the_origin_aborts() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.put(66, &elevator("white"));
        world.solid.insert(64);
        assert!(plan(&mut world, 64.0, true, false).is_none());
    }

    #[test]
    fn the_nearest_target_upwards_wins() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.put(66, &elevator("white"));
        world.put(69, &elevator("white"));
        let target = plan(&mut world, 64.0, true, false).unwrap();
        assert_eq!(target.elevator_y, 66, "nearest, not farthest");
        assert_eq!(target.feet_y, 67.0);
    }

    #[test]
    fn blocked_cells_are_skipped_and_the_scan_continues() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.put(66, &elevator("white"));
        world.put(69, &elevator("white"));
        world.solid.insert(67); // suffocating above 66
        let target = plan(&mut world, 64.0, true, false).unwrap();
        assert_eq!(target.elevator_y, 69);
    }

    #[test]
    fn the_range_cap_is_inclusive() {
        let mut world = Column::new();
        world.put(0, &elevator("white"));
        world.put(RANGE, &elevator("white"));
        assert!(plan(&mut world, 1.0, true, false).is_some());
        let mut world = Column::new();
        world.put(0, &elevator("white"));
        world.put(RANGE + 1, &elevator("white"));
        assert!(plan(&mut world, 1.0, true, false).is_none());
    }

    #[test]
    fn same_color_skips_other_dyes() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.put(66, &elevator("red"));
        world.put(69, &elevator("white"));
        assert_eq!(plan(&mut world, 64.0, true, true).unwrap().elevator_y, 69);
        assert_eq!(plan(&mut world, 64.0, true, false).unwrap().elevator_y, 66);
    }

    #[test]
    fn sneaking_travels_downwards() {
        let mut world = Column::new();
        world.put(63, &elevator("white"));
        world.put(57, &elevator("white"));
        let target = plan(&mut world, 64.0, false, false).unwrap();
        assert_eq!(target.elevator_y, 57);
        assert_eq!(target.feet_y, 58.0);
    }

    #[test]
    fn the_destination_is_centred_on_the_block() {
        assert_eq!(travel_position(0, 0, 67.0), [0.5, 67.0, 0.5]);
        assert_eq!(travel_position(-3, 12, 70.0), [-2.5, 70.0, 12.5]);
    }

    #[test]
    fn no_elevator_means_no_travel() {
        let mut world = Column::new();
        world.put(63, "minecraft:stone");
        assert!(plan(&mut world, 64.0, true, false).is_none());
    }
}
