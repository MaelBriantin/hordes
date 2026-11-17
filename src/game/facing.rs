use super::Vec2;

/// How much more one axis of a move must count than the other to turn a character from one
/// way to the other: a near diagonal keeps the way it had, so the character doesn't flicker.
const TURN_MARGIN: f32 = 1.25;

/// The way a character faces: where it last went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facing {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Facing {
    /// The way behind.
    pub fn opposite(self) -> Self {
        match self {
            Facing::Up => Facing::Down,
            Facing::Down => Facing::Up,
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
        }
    }

    /// The way it points, length 1.
    pub fn way(self) -> Vec2 {
        match self {
            Facing::Up => Vec2::new(0.0, -1.0),
            Facing::Down => Vec2::new(0.0, 1.0),
            Facing::Left => Vec2::new(-1.0, 0.0),
            Facing::Right => Vec2::new(1.0, 0.0),
        }
    }

    /// The way a character faces after moving in `direction`. Still, it keeps its way; moving
    /// its way, even in a diagonal, too, unless the other axis clearly counts more. Otherwise it
    /// turns to the axis that counts more, sideways on a tie.
    pub fn after(self, direction: Vec2) -> Self {
        if direction == Vec2::ZERO {
            return self;
        }
        let (along, across) = match self {
            Facing::Up => (-direction.y, direction.x),
            Facing::Down => (direction.y, direction.x),
            Facing::Left => (-direction.x, direction.y),
            Facing::Right => (direction.x, direction.y),
        };
        if along > 0.0 && along * TURN_MARGIN >= across.abs() {
            self
        } else if direction.x.abs() >= direction.y.abs() {
            if direction.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            }
        } else if direction.y > 0.0 {
            Facing::Down
        } else {
            Facing::Up
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_still_keeps_the_way() {
        assert_eq!(Facing::Left.after(Vec2::ZERO), Facing::Left);
    }

    #[test]
    fn a_straight_move_turns_that_way() {
        assert_eq!(Facing::Down.after(Vec2::new(-1.0, 0.0)), Facing::Left);
        assert_eq!(Facing::Left.after(Vec2::new(0.0, -1.0)), Facing::Up);
        assert_eq!(Facing::Left.after(Vec2::new(1.0, 0.0)), Facing::Right);
    }

    #[test]
    fn a_diagonal_keeps_the_way_it_goes() {
        assert_eq!(Facing::Up.after(Vec2::new(1.0, -1.0)), Facing::Up);
        assert_eq!(Facing::Right.after(Vec2::new(1.0, -1.0)), Facing::Right);
    }

    #[test]
    fn a_diagonal_from_elsewhere_turns_sideways() {
        assert_eq!(Facing::Down.after(Vec2::new(1.0, -1.0)), Facing::Right);
    }

    #[test]
    fn a_near_diagonal_keeps_the_way_and_a_clear_turn_does_not() {
        assert_eq!(Facing::Right.after(Vec2::new(0.7, 0.8)), Facing::Right);
        assert_eq!(Facing::Right.after(Vec2::new(0.3, 0.95)), Facing::Down);
    }
}
