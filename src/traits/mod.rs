use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    // Combat
    Strong,
    Weak,
    Tough,
    Fragile,
    Berserker,
    Cowardly,
    Brave,
    Tactical,
    Duelist,
    Bloodthirsty,
    // Cognitive
    Intelligent,
    SlowMinded,
    Strategic,
    Impulsive,
    Adaptive,
    Forgetful,
    Curious,
    Paranoid,
    // Biological
    Fast,
    Slow,
    Regenerative,
    Frail,
    Giant,
    Tiny,
    Immortal,
    ShortLived,
    Fertile,
    Sterile,
    // Social
    Loyal,
    Rebel,
    Leader,
    Follower,
    Charismatic,
    Isolated,
    // Environmental
    Fireproof,
    Aquatic,
    HeatResistant,
    ColdResistant,
    PoisonResistant,
    Corrupted,
    // Special
    Blessed,
    Cursed,
    Lucky,
    Unlucky,
    ArcaneTouched,
    WorldAnchor,
}

impl Trait {
    pub fn all() -> &'static [Trait] {
        &[
            Trait::Strong, Trait::Weak, Trait::Tough, Trait::Fragile,
            Trait::Berserker, Trait::Cowardly, Trait::Brave, Trait::Tactical,
            Trait::Duelist, Trait::Bloodthirsty,
            Trait::Intelligent, Trait::SlowMinded, Trait::Strategic, Trait::Impulsive,
            Trait::Adaptive, Trait::Forgetful, Trait::Curious, Trait::Paranoid,
            Trait::Fast, Trait::Slow, Trait::Regenerative, Trait::Frail,
            Trait::Giant, Trait::Tiny, Trait::Immortal, Trait::ShortLived,
            Trait::Fertile, Trait::Sterile,
            Trait::Loyal, Trait::Rebel, Trait::Leader, Trait::Follower,
            Trait::Charismatic, Trait::Isolated,
            Trait::Fireproof, Trait::Aquatic, Trait::HeatResistant, Trait::ColdResistant,
            Trait::PoisonResistant, Trait::Corrupted,
            Trait::Blessed, Trait::Cursed, Trait::Lucky, Trait::Unlucky,
            Trait::ArcaneTouched, Trait::WorldAnchor,
        ]
    }

    pub fn rarity(&self) -> f32 {
        match self {
            Trait::Blessed | Trait::Cursed | Trait::ArcaneTouched | Trait::WorldAnchor => 0.01,
            Trait::Immortal | Trait::Giant | Trait::Berserker => 0.03,
            Trait::Lucky | Trait::Unlucky => 0.05,
            _ => 0.15,
        }
    }

    pub fn contradicts(&self, other: &Trait) -> bool {
        self.contradiction_pair(other) || other.contradiction_pair(self)
    }

    fn contradiction_pair(&self, other: &Trait) -> bool {
        matches!(
            (self, other),
            (Trait::Strong, Trait::Weak)
                | (Trait::Tough, Trait::Fragile)
                | (Trait::Brave, Trait::Cowardly)
                | (Trait::Intelligent, Trait::SlowMinded)
                | (Trait::Fast, Trait::Slow)
                | (Trait::Giant, Trait::Tiny)
                | (Trait::Immortal, Trait::ShortLived)
                | (Trait::Fertile, Trait::Sterile)
                | (Trait::Loyal, Trait::Rebel)
                | (Trait::Leader, Trait::Follower)
                | (Trait::Charismatic, Trait::Isolated)
                | (Trait::Lucky, Trait::Unlucky)
                | (Trait::Blessed, Trait::Cursed)
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Trait::Strong => "Strong",
            Trait::Weak => "Weak",
            Trait::Tough => "Tough",
            Trait::Fragile => "Fragile",
            Trait::Berserker => "Berserker",
            Trait::Cowardly => "Cowardly",
            Trait::Brave => "Brave",
            Trait::Tactical => "Tactical",
            Trait::Duelist => "Duelist",
            Trait::Bloodthirsty => "Bloodthirsty",
            Trait::Intelligent => "Intelligent",
            Trait::SlowMinded => "Slow-Minded",
            Trait::Strategic => "Strategic",
            Trait::Impulsive => "Impulsive",
            Trait::Adaptive => "Adaptive",
            Trait::Forgetful => "Forgetful",
            Trait::Curious => "Curious",
            Trait::Paranoid => "Paranoid",
            Trait::Fast => "Fast",
            Trait::Slow => "Slow",
            Trait::Regenerative => "Regenerative",
            Trait::Frail => "Frail",
            Trait::Giant => "Giant",
            Trait::Tiny => "Tiny",
            Trait::Immortal => "Immortal",
            Trait::ShortLived => "Short-Lived",
            Trait::Fertile => "Fertile",
            Trait::Sterile => "Sterile",
            Trait::Loyal => "Loyal",
            Trait::Rebel => "Rebel",
            Trait::Leader => "Leader",
            Trait::Follower => "Follower",
            Trait::Charismatic => "Charismatic",
            Trait::Isolated => "Isolated",
            Trait::Fireproof => "Fireproof",
            Trait::Aquatic => "Aquatic",
            Trait::HeatResistant => "Heat-Resistant",
            Trait::ColdResistant => "Cold-Resistant",
            Trait::PoisonResistant => "Poison-Resistant",
            Trait::Corrupted => "Corrupted",
            Trait::Blessed => "Blessed",
            Trait::Cursed => "Cursed",
            Trait::Lucky => "Lucky",
            Trait::Unlucky => "Unlucky",
            Trait::ArcaneTouched => "Arcane-Touched",
            Trait::WorldAnchor => "World-Anchor",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Trait::Blessed | Trait::Lucky => Color::srgb(1.0, 0.85, 0.0),
            Trait::Cursed | Trait::Unlucky | Trait::Corrupted => Color::srgb(0.5, 0.0, 0.5),
            Trait::Immortal | Trait::WorldAnchor | Trait::ArcaneTouched => Color::srgb(0.3, 0.8, 1.0),
            Trait::Strong | Trait::Giant | Trait::Berserker => Color::srgb(0.9, 0.2, 0.1),
            Trait::Leader | Trait::Charismatic => Color::srgb(1.0, 0.65, 0.0),
            _ => Color::srgb(0.7, 0.7, 0.7),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct TraitSet {
    pub traits: SmallVec<[Trait; 8]>,
}

impl TraitSet {
    pub fn new() -> Self {
        Self { traits: SmallVec::new() }
    }

    pub fn add(&mut self, t: Trait) {
        self.traits.retain(|existing| !t.contradicts(existing));
        if !self.traits.contains(&t) {
            self.traits.push(t);
        }
    }

    pub fn has(&self, t: Trait) -> bool {
        self.traits.contains(&t)
    }

    pub fn damage_modifier(&self) -> f32 {
        let mut m = 1.0;
        for t in &self.traits {
            match t {
                Trait::Strong => m *= 1.5,
                Trait::Weak => m *= 0.6,
                Trait::Giant => m *= 1.3,
                Trait::Tiny => m *= 0.7,
                Trait::Blessed => m *= 1.2,
                Trait::Cursed => m *= 0.85,
                _ => {}
            }
        }
        m
    }

    pub fn defense_modifier(&self) -> f32 {
        let mut m = 1.0;
        for t in &self.traits {
            match t {
                Trait::Tough => m *= 1.5,
                Trait::Fragile => m *= 0.6,
                Trait::Giant => m *= 1.2,
                Trait::WorldAnchor => m *= 2.0,
                _ => {}
            }
        }
        m
    }

    pub fn speed_modifier(&self) -> f32 {
        let mut m = 1.0;
        for t in &self.traits {
            match t {
                Trait::Fast => m *= 1.5,
                Trait::Slow => m *= 0.6,
                Trait::Giant => m *= 0.85,
                Trait::Tiny => m *= 1.2,
                _ => {}
            }
        }
        m
    }

    pub fn aggression_modifier(&self) -> f32 {
        let mut m = 0.0;
        for t in &self.traits {
            match t {
                Trait::Bloodthirsty => m += 0.4,
                Trait::Berserker => m += 0.3,
                Trait::Brave => m += 0.1,
                Trait::Cowardly => m -= 0.3,
                Trait::Strategic => m -= 0.1,
                _ => {}
            }
        }
        m
    }

    pub fn fertility_modifier(&self) -> f32 {
        let mut m = 1.0;
        for t in &self.traits {
            match t {
                Trait::Fertile => m *= 2.0,
                Trait::Sterile => m *= 0.0,
                _ => {}
            }
        }
        m
    }

    pub fn generate_random(rng: &mut impl Rng, count: usize) -> Self {
        let mut set = Self::new();
        let all = Trait::all();
        for _ in 0..count {
            let t = all[rng.gen_range(0..all.len())];
            if rng.gen::<f32>() < t.rarity() * 5.0 {
                set.add(t);
            }
        }
        set
    }

    pub fn inherit(parent_a: &TraitSet, parent_b: &TraitSet, rng: &mut impl Rng) -> Self {
        let mut set = Self::new();
        for t in &parent_a.traits {
            if rng.gen::<f32>() < 0.5 {
                set.add(*t);
            }
        }
        for t in &parent_b.traits {
            if rng.gen::<f32>() < 0.5 {
                set.add(*t);
            }
        }
        if rng.gen::<f32>() < 0.03 {
            let all = Trait::all();
            set.add(all[rng.gen_range(0..all.len())]);
        }
        set
    }
}
