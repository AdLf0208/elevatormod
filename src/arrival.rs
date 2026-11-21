//! The look a ride leaves the player with.
//!
//! Upstream's server half (`TeleportPacket.java:71-77`) touches the player's view in exactly two
//! places: a **directional** target elevator replaces the yaw with its own `facing`, and the pitch
//! is levelled; a plain elevator keeps both angles. The two config switches behind that are
//! `resetPitchDirectional` (default `true`) and `resetPitchNormal` (default `false`,
//! `Config.java:53-58`).
//!
//! This module is the rule alone: no `mod_api` types, so it is unit-tested here and the glue only
//! reads the target block's `directional`/`facing` states and hands them over.

/// `resetPitchDirectional`: a directional landing levels the pitch (upstream default).
pub const RESET_PITCH_DIRECTIONAL: bool = true;

/// `resetPitchNormal`: a plain landing keeps the pitch the player arrived with (upstream default).
pub const RESET_PITCH_NORMAL: bool = false;

/// One of the four horizontal directions a directional elevator can face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    North,
    South,
    West,
    East,
}

impl Facing {
    /// The direction a state value spells (`north`, `south`, `west`, `east`), or None for
    /// anything else: an unreadable state must not invent a direction.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "east" => Some(Self::East),
            _ => None,
        }
    }

    /// The yaw that faces this way, as vanilla's `Direction.toYRot` gives it
    /// (south 0, west 90, north 180, east 270).
    pub fn yaw(self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 90.0,
            Self::North => 180.0,
            Self::East => 270.0,
        }
    }
}

/// A player's view, in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

/// The view a ride lands with. `facing` is `Some` when the target elevator is directional, and
/// `None` for a plain one, which keeps the view it arrived with.
pub fn arrival(facing: Option<Facing>, current: Look) -> Look {
    let (yaw, reset_pitch) = match facing {
        Some(facing) => (facing.yaw(), RESET_PITCH_DIRECTIONAL),
        None => (current.yaw, RESET_PITCH_NORMAL),
    };
    Look {
        yaw,
        pitch: if reset_pitch { 0.0 } else { current.pitch },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARRIVING: Look = Look {
        yaw: 33.0,
        pitch: 40.0,
    };

    #[test]
    fn a_plain_elevator_keeps_the_view() {
        assert_eq!(
            arrival(None, ARRIVING),
            ARRIVING,
            "a non-directional elevator must not turn or level the player"
        );
    }

    #[test]
    fn a_directional_elevator_turns_you_to_its_face_and_levels_the_pitch() {
        let yaw = |facing| arrival(Some(facing), ARRIVING).yaw;
        assert_eq!(yaw(Facing::South), 0.0);
        assert_eq!(yaw(Facing::West), 90.0);
        assert_eq!(yaw(Facing::North), 180.0);
        assert_eq!(yaw(Facing::East), 270.0);
        assert_eq!(
            arrival(Some(Facing::East), ARRIVING).pitch,
            0.0,
            "resetPitchDirectional is on: the pitch is levelled"
        );
    }

    #[test]
    fn the_angle_you_jumped_with_does_not_move_a_directional_landing() {
        let from_north = Look {
            yaw: 179.0,
            pitch: -12.0,
        };
        let from_south = Look {
            yaw: -179.0,
            pitch: 88.0,
        };
        assert_eq!(arrival(Some(Facing::West), from_north).yaw, 90.0);
        assert_eq!(arrival(Some(Facing::West), from_south).yaw, 90.0);
    }

    #[test]
    fn the_state_spellings_are_the_four_facings() {
        assert_eq!(Facing::parse("north"), Some(Facing::North));
        assert_eq!(Facing::parse("south"), Some(Facing::South));
        assert_eq!(Facing::parse("west"), Some(Facing::West));
        assert_eq!(Facing::parse("east"), Some(Facing::East));
        assert_eq!(
            Facing::parse("up"),
            None,
            "a vertical facing is not readable"
        );
        assert_eq!(Facing::parse(""), None);
    }
}
