//! The grenades in the air: a dark olive body arcing to where it blows, its fuse sparking.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::{GRENADE_ARC, Vec2, World};

const BODY: Color = Color::Rgb(60, 80, 50);
const SPARK: [Color; 2] = [Color::Rgb(255, 225, 100), Color::Rgb(245, 130, 40)];

impl Canvas<'_> {
    /// The grenades in the air, each arcing up then down to where it blows, its fuse sparking;
    /// the spitters' globs of acid too, lobbed as they are.
    pub(super) fn paint_grenades(&mut self, world: &World, frame: u32) {
        self.paint_globs(world);
        for thrown in &world.grenades {
            let progress = thrown.progress();
            let along = thrown.from + (thrown.to - thrown.from) * progress;
            let lift = 4.0 * progress * (1.0 - progress) * GRENADE_ARC;
            let (x, y) = pixel(along - Vec2::new(0.0, lift));
            self.set((x, y), BODY);
            self.set((x, y - 1), SPARK[usize::try_from(frame % 2).unwrap_or(0)]);
        }
    }
}
