//! The scoped settings that can be controlled at runtime for the epoch GCSetting
//!
//! # GC State
//!
//! Users can control whether the GC will run, and whether deeper scopes
//! can even re-enable the GC

use core::fmt::Debug;
use std::cell::Cell;
use std::ops::Deref;


/// Determines the strength of the setting
#[derive(Copy, Clone, Debug)]
pub enum Strength<T: Stronger + Copy + Clone + Debug> {
    /// Deeper scopes can change the setting at will
    Lenient,

    /// Deeper scopes can only strengthen the setting from it's set value
    AsStrongAs(T),

    /// Deeper scopes cannot change the setting
    Strict
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Collect {
    NoCollect,
    Collect,
}

pub trait Stronger {
    /// Returns if self is strong than the other
    fn stronger_than(&self, &Self) -> bool;
}

impl<T: Stronger + Copy + Clone + Debug> Stronger for Strength<T> {
    fn stronger_than(&self, other: &Self) -> bool {
        match self {
            &Strength::Lenient => false,
            &Strength::AsStrongAs(val) => {
                match other {
                    &Strength::Lenient => true,
                    &Strength::AsStrongAs(ref test) => val.stronger_than(test),
                    &Strength::Strict => false,
                }
            }
            &Strength::Strict => true,
        }
    }
}

impl Stronger for Collect {
    #[inline]
    fn stronger_than(&self, _: &Self) -> bool {
        // If we're at collect, any change is the same or stronger
        // If we're at nocollect, and change is the same or weaker
        match *self {
            Collect::Collect => false,
            Collect::NoCollect => true,
        }
    }
}

#[inline]
fn strongest<T: Stronger>(old: T, new: T) -> T {
    if old.stronger_than(&new) { old } else { new }
}

#[derive(Clone, Copy, Debug)]
pub struct Setting<T: Stronger + Copy + Clone + Debug> {
    val: T,
    strength: Strength<T>,
}

impl<T: Stronger + Clone + Copy + Debug> Deref for Setting<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.val
    }
}

macro_rules! generate_setting_fncs {
    ($s:ident, $v:ident, $va: ident, $st:ident) => {{
        let mut setting = $s.old.$v.get();
        match setting.strength {
            Strength::Lenient => {
                setting.val = $va;
                setting.strength = $st;
            },
            Strength::AsStrongAs(test) => {
                setting.val = strongest($va, test);
                setting.strength = strongest(setting.strength, $st);
            },
            Strength::Strict => {},
        }
        $s.cur.collect.set(setting);
        $s
    }};
    ($s:ident, $v:ident, $va: ident) => {{
        let mut setting = $s.old.$v.get();
        match setting.strength {
            Strength::Lenient => {
                setting.val = $va;
            },
            Strength::AsStrongAs(test) => {
                setting.val = strongest($va, test);
            },
            Strength::Strict => {},
        }
        $s.cur.collect.set(setting);
        $s
    }};
}

/// This struct is a collection of available settings with a builder api
#[derive(Clone, Debug)]
pub struct GCSettings {
    pub collect: Cell<Setting<Collect>>
}

impl GCSettings {
    pub fn new() -> GCSettings {
        GCSettings {
           collect: Cell::new(Setting {
                val: Collect::Collect,
                strength: Strength::Lenient,
            })
        }
    }
}

pub struct ScopedGCSettings<'a> {
    old: GCSettings,
    cur: &'a GCSettings
}

impl<'a> ScopedGCSettings<'a> {
   pub fn new(old: &'a GCSettings) -> ScopedGCSettings<'a> {
        ScopedGCSettings {
            old: old.clone(),
            cur: old,
        }
    }

    pub fn with_collect_strength(&'a self, val: Collect,
                                strength: Strength<Collect>)
                             -> &ScopedGCSettings<'a> {
        generate_setting_fncs!(self, collect, val, strength)
    }

    pub fn with_collect(&'a self, val: Collect) -> &ScopedGCSettings<'a> {
        generate_setting_fncs!(self, collect, val)
    }

}
