//! Weapon specifications (`weapon.dat`), 13-slot San Andreas inventory,
//! reloading, damage, and hitscan/explosive combat definitions.

use gtasarust_gta_formats::dat::WeaponDat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Unarmed,
    Melee,
    Handgun,
    Shotgun,
    SubmachineGun,
    AssaultRifle,
    SniperRifle,
    Heavy,
    Thrown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponSpec {
    pub id: u32,
    pub name: String,
    pub slot: usize,
    pub kind: WeaponKind,
    pub damage: f32,
    pub range: f32,
    pub fire_interval: f32,
    pub clip_size: u32,
    pub reload_time: f32,
    pub is_explosive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedWeapon {
    pub spec: WeaponSpec,
    pub ammo_in_clip: u32,
    pub reserve_ammo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponInventory {
    pub slots: [Option<OwnedWeapon>; 13],
    pub active_slot: usize,
    pub fire_cooldown: f32,
    pub reload_timer: f32,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        let mut slots: [Option<OwnedWeapon>; 13] = Default::default();
        slots[0] = Some(OwnedWeapon {
            spec: WeaponSpec {
                id: 0,
                name: "Fist".to_string(),
                slot: 0,
                kind: WeaponKind::Unarmed,
                damage: 12.0,
                range: 2.2,
                fire_interval: 0.35,
                clip_size: 0,
                reload_time: 0.0,
                is_explosive: false,
            },
            ammo_in_clip: 0,
            reserve_ammo: 0,
        });
        Self {
            slots,
            active_slot: 0,
            fire_cooldown: 0.0,
            reload_timer: 0.0,
        }
    }
}

impl WeaponInventory {
    pub fn give_weapon(&mut self, spec: WeaponSpec, total_ammo: u32) {
        let slot = spec.slot.min(12);
        let clip = if spec.clip_size == 0 {
            0
        } else {
            total_ammo.min(spec.clip_size)
        };
        let reserve = total_ammo.saturating_sub(clip);
        self.slots[slot] = Some(OwnedWeapon {
            spec,
            ammo_in_clip: clip,
            reserve_ammo: reserve,
        });
    }

    pub fn active(&self) -> Option<&OwnedWeapon> {
        self.slots[self.active_slot].as_ref()
    }

    pub fn cycle_next(&mut self) {
        for offset in 1..=13 {
            let idx = (self.active_slot + offset) % 13;
            if self.slots[idx].is_some() {
                self.active_slot = idx;
                self.reload_timer = 0.0;
                break;
            }
        }
    }

    pub fn cycle_prev(&mut self) {
        for offset in 1..=13 {
            let idx = (self.active_slot + 13 - offset) % 13;
            if self.slots[idx].is_some() {
                self.active_slot = idx;
                self.reload_timer = 0.0;
                break;
            }
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        }
        if self.reload_timer > 0.0 {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.reload_timer = 0.0;
                if let Some(w) = &mut self.slots[self.active_slot]
                    && w.spec.clip_size > 0
                {
                    let needed = w.spec.clip_size.saturating_sub(w.ammo_in_clip);
                    let transfer = needed.min(w.reserve_ammo);
                    w.ammo_in_clip += transfer;
                    w.reserve_ammo -= transfer;
                }
            }
        }
    }

    /// Attempt to fire the active weapon; returns damage and range if fired.
    pub fn try_fire(&mut self) -> Option<(f32, f32, bool)> {
        if self.fire_cooldown > 0.0 || self.reload_timer > 0.0 {
            return None;
        }
        let w = self.slots[self.active_slot].as_mut()?;
        if w.spec.clip_size > 0 {
            if w.ammo_in_clip == 0 {
                if w.reserve_ammo > 0 {
                    self.reload_timer = w.spec.reload_time;
                }
                return None;
            }
            w.ammo_in_clip -= 1;
            if w.ammo_in_clip == 0 && w.reserve_ammo > 0 {
                self.reload_timer = w.spec.reload_time;
            }
        }
        self.fire_cooldown = w.spec.fire_interval;
        Some((w.spec.damage, w.spec.range, w.spec.is_explosive))
    }

    pub fn preset_colt45() -> WeaponSpec {
        WeaponSpec {
            id: 22,
            name: "9mm".to_string(),
            slot: 2,
            kind: WeaponKind::Handgun,
            damage: 25.0,
            range: 35.0,
            fire_interval: 0.28,
            clip_size: 17,
            reload_time: 1.2,
            is_explosive: false,
        }
    }

    pub fn from_weapon_dat(dat: &WeaponDat, name: &str) -> Option<WeaponSpec> {
        let e = dat.weapons.get(&name.to_ascii_uppercase())?;
        Some(WeaponSpec {
            id: e.model_id.max(0) as u32,
            name: e.name.clone(),
            slot: (e.slot as usize).min(12),
            kind: if e.fire_type == "MELEE" {
                WeaponKind::Melee
            } else {
                WeaponKind::Handgun
            },
            damage: e.damage as f32,
            range: e.weapon_range,
            fire_interval: 0.25,
            clip_size: e.clip_size,
            reload_time: 1.2,
            is_explosive: e.fire_type == "PROJECTILE",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handgun_fires_and_reloads() {
        let mut inv = WeaponInventory::default();
        inv.give_weapon(WeaponInventory::preset_colt45(), 34);
        inv.cycle_next();
        for _ in 0..17 {
            assert!(inv.try_fire().is_some());
            inv.tick(0.3);
        }
        // Clip empty -> reload started automatically, next tick finishes reload
        assert!(inv.try_fire().is_none());
        inv.tick(1.3);
        assert_eq!(inv.active().unwrap().ammo_in_clip, 17);
    }
}
