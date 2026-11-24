//! The player's torch: when it lights, flickers and holds, what it lights, and the dark when it is out.

use super::{GOGGLES_OUT_TICKS, RngExt, TORCH_FADE, TORCH_FLICKER_MOST, World, torch_lights_up};

impl World {
    /// How far around the player the night is lit, in tiles: their light, less what the
    /// weather takes, never under 4.
    pub fn light_radius(&self) -> f32 {
        (self.player.build.light_radius() - self.weather().light_loss()).max(4.0)
    }

    /// Where Glare slows the enemies, and how much, in percent: around the player, as far as
    /// their torch's light shows, fading included, as a campfire heals; none without Glare, or
    /// the torch not lit.
    pub(super) fn glare(&self) -> Option<(f32, u32)> {
        let percent = self.player.build.glare_percent();
        (percent > 0 && self.torch_lit()).then(|| (self.light_radius() + TORCH_FADE, percent))
    }

    /// Whether the player's torch is lit, and holds: not at dusk; once the player lights it, after
    /// flickering a while.
    pub fn torch_lit(&self) -> bool {
        let Some(true) = self.torch_on() else {
            return false;
        };
        self.ticks >= self.torch_steady_at
    }

    /// Whether the torch shines now: lit, or flickering as it lights up, on and off as luck has
    /// it. Flickering, it lights nothing for the run: the player isn't seen, the guns hold.
    pub fn torch_shines(&self) -> bool {
        if self.torch_lit() {
            return true;
        }
        let flickering = self.torch_on() == Some(true) && self.ticks < self.torch_steady_at;
        let luck = ((self.ticks / 2) ^ self.torch_luck)
            .wrapping_mul(0x9E37_79B9)
            .rotate_left(7);
        flickering && luck.is_multiple_of(3)
    }

    /// The torch starts to light up: it flickers, as long as luck has it (maybe not at all),
    /// before it holds.
    pub(super) fn light_up_torch(&mut self) {
        self.torch_steady_at =
            self.ticks.max(torch_lights_up()) + self.rng.random_range(0..=TORCH_FLICKER_MOST);
        self.torch_luck = self.rng.random();
    }

    /// A strike of lightning with the player in its ring puts their torch out: up to them to light
    /// it again, quick, flickering as it does, F shown to them again the first time; the
    /// infiltrator's goggles go blank a while too.
    pub(super) fn cut_torch(&mut self) {
        if self.torch_on() == Some(true) {
            self.torch_switch = false;
            self.relight_hint = !self.torch_cut_once;
            self.torch_cut_once = true;
        }
        self.goggles_out = GOGGLES_OUT_TICKS;
    }

    /// Whether the infiltrator sees in the dark: their torch out, their goggles not blanked by
    /// lightning.
    pub fn night_vision(&self) -> bool {
        self.player.build.hero().perk().ghost()
            && self.torch_on() == Some(false)
            && self.goggles_out == 0
    }

    /// Whether F is shown to the player under them: the night fallen, their torch never lit yet
    /// this run, or put out by lightning the first time and not lit again; never to the
    /// infiltrator, at home in the dark.
    pub fn torch_hint(&self) -> bool {
        self.torch_on() == Some(false)
            && (!self.torch_lit_once || self.relight_hint)
            && !self.player.build.hero().perk().ghost()
    }

    /// Whether the player has their torch on: none before the night falls.
    pub fn torch_on(&self) -> Option<bool> {
        (self.ticks >= torch_lights_up()).then_some(self.torch_switch)
    }

    /// Puts the torch out, or lights it, flickering: once the night has fallen.
    pub fn switch_torch(&mut self) {
        match self.torch_on() {
            None => {}
            Some(true) => self.torch_switch = false,
            Some(false) => {
                self.torch_switch = true;
                self.torch_lit_once = true;
                self.relight_hint = false;
                self.light_up_torch();
            }
        }
    }

    /// Whether the enemies see the player from afar: by daylight, or by their torch.
    pub(super) fn seen_from_afar(&self) -> bool {
        self.torch_on().is_none() || self.torch_lit()
    }
}
