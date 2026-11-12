use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Directives parsed from `data/gta.dat` and `data/default.dat`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GtaDat {
    pub img_archives: Vec<String>,
    pub ide_files: Vec<String>,
    pub ipl_files: Vec<String>,
    pub col_files: Vec<String>,
    pub model_files: Vec<String>,
    pub txd_files: Vec<String>,
}

impl GtaDat {
    pub fn parse(text: &str) -> Self {
        let mut out = Self::default();
        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(cmd) = parts.next() else {
                continue;
            };
            let rest = parts.collect::<Vec<_>>().join(" ");
            match cmd.to_ascii_uppercase().as_str() {
                "IMG" => out.img_archives.push(rest),
                "IDE" => out.ide_files.push(rest),
                "IPL" => out.ipl_files.push(rest),
                "COLFILE" => {
                    // Format: COLFILE <level> <path>
                    if let Some((_, path)) = rest.split_once(' ') {
                        out.col_files.push(path.trim().to_string());
                    }
                }
                "MODELFILE" => out.model_files.push(rest),
                "TEXDICTION" => out.txd_files.push(rest),
                _ => {}
            }
        }
        out
    }
}

/// Parsed vehicle entry from `data/handling.cfg`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleHandling {
    pub identifier: String,
    pub mass: f32,
    pub turn_mass: f32,
    pub drag_mult: f32,
    pub center_of_mass: [f32; 3],
    pub percent_submerged: u32,
    pub traction_multiplier: f32,
    pub traction_loss: f32,
    pub traction_bias: f32,
    pub num_gears: u8,
    pub max_velocity: f32,
    pub engine_acceleration: f32,
    pub engine_inertia: f32,
    pub drive_type: char,
    pub engine_type: char,
    pub brake_deceleration: f32,
    pub brake_bias: f32,
    pub abs: bool,
    pub steering_lock: f32,
    pub suspension_force: f32,
    pub suspension_damping: f32,
    pub suspension_high_speed_damping: f32,
    pub suspension_upper_limit: f32,
    pub suspension_lower_limit: f32,
    pub suspension_bias: f32,
    pub suspension_anti_dive: f32,
    pub seat_offset_distance: f32,
    pub collision_damage_mult: f32,
    pub monetary_value: u32,
    pub model_flags: u32,
    pub handling_flags: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandlingCfg {
    pub vehicles: HashMap<String, VehicleHandling>,
}

impl HandlingCfg {
    pub fn parse(text: &str) -> Self {
        let mut vehicles = HashMap::new();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty()
                || line.starts_with(';')
                || line.starts_with('#')
                || line.starts_with('%')
                || line.starts_with('$')
                || line.starts_with('!')
                || line.starts_with('^')
            {
                continue;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 34 {
                continue;
            }
            let Ok(mass) = cols[1].parse::<f32>() else {
                continue;
            };
            let id = cols[0].to_ascii_uppercase();
            let entry = VehicleHandling {
                identifier: id.clone(),
                mass,
                turn_mass: cols[2].parse().unwrap_or(mass * 2.0),
                drag_mult: cols[3].parse().unwrap_or(2.0),
                center_of_mass: [
                    cols[4].parse().unwrap_or(0.0),
                    cols[5].parse().unwrap_or(0.0),
                    cols[6].parse().unwrap_or(0.0),
                ],
                percent_submerged: cols[7].parse().unwrap_or(75),
                traction_multiplier: cols[8].parse().unwrap_or(0.75),
                traction_loss: cols[9].parse().unwrap_or(0.85),
                traction_bias: cols[10].parse().unwrap_or(0.5),
                num_gears: cols[11].parse().unwrap_or(5),
                max_velocity: cols[12].parse().unwrap_or(160.0),
                engine_acceleration: cols[13].parse().unwrap_or(20.0),
                engine_inertia: cols[14].parse().unwrap_or(10.0),
                drive_type: cols[15].chars().next().unwrap_or('R'),
                engine_type: cols[16].chars().next().unwrap_or('P'),
                brake_deceleration: cols[17].parse().unwrap_or(8.0),
                brake_bias: cols[18].parse().unwrap_or(0.5),
                abs: cols[19].parse::<u32>().unwrap_or(0) != 0,
                steering_lock: cols[20].parse().unwrap_or(35.0),
                suspension_force: cols[21].parse().unwrap_or(1.5),
                suspension_damping: cols[22].parse().unwrap_or(0.1),
                suspension_high_speed_damping: cols[23].parse().unwrap_or(0.0),
                suspension_upper_limit: cols[24].parse().unwrap_or(0.3),
                suspension_lower_limit: cols[25].parse().unwrap_or(-0.15),
                suspension_bias: cols[26].parse().unwrap_or(0.5),
                suspension_anti_dive: cols[27].parse().unwrap_or(0.2),
                seat_offset_distance: cols[28].parse().unwrap_or(0.25),
                collision_damage_mult: cols[29].parse().unwrap_or(0.4),
                monetary_value: cols[30].parse().unwrap_or(20000),
                model_flags: u32::from_str_radix(cols[31], 16).unwrap_or(0),
                handling_flags: u32::from_str_radix(cols[32], 16).unwrap_or(0),
            };
            vehicles.insert(id, entry);
        }
        Self { vehicles }
    }
}

/// Water quad parsed from `data/water.dat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterQuad {
    pub corners: [[f32; 3]; 4],
    pub flags: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaterDat {
    pub quads: Vec<WaterQuad>,
}

impl WaterDat {
    pub fn parse(text: &str) -> Self {
        let mut quads = Vec::new();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("processed") || line.starts_with('#') {
                continue;
            }
            let nums: Vec<f32> = line
                .split_whitespace()
                .filter_map(|t| t.parse::<f32>().ok())
                .collect();
            if nums.len() >= 29 {
                let flags = nums[28] as u32;
                quads.push(WaterQuad {
                    corners: [
                        [nums[0], nums[1], nums[2]],
                        [nums[7], nums[8], nums[9]],
                        [nums[14], nums[15], nums[16]],
                        [nums[21], nums[22], nums[23]],
                    ],
                    flags,
                });
            }
        }
        Self { quads }
    }
}

/// Atmospheric/lighting entry parsed from `data/timecyc.dat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimecycEntry {
    pub ambient_rgb: [u8; 3],
    pub ambient_obj_rgb: [u8; 3],
    pub sky_top_rgb: [u8; 3],
    pub sky_bottom_rgb: [u8; 3],
    pub sun_core_rgb: [u8; 3],
    pub sun_corona_rgb: [u8; 3],
    pub fog_start: f32,
    pub far_clip: f32,
    pub water_rgba: [f32; 4],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecycDat {
    pub entries: Vec<TimecycEntry>,
}

impl TimecycDat {
    pub fn parse(text: &str) -> Self {
        let mut entries = Vec::new();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 40 {
                continue;
            }
            let p_u8 = |i: usize| cols[i].parse::<f32>().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
            let p_f32 = |i: usize| cols[i].parse::<f32>().unwrap_or(0.0);
            entries.push(TimecycEntry {
                ambient_rgb: [p_u8(0), p_u8(1), p_u8(2)],
                ambient_obj_rgb: [p_u8(3), p_u8(4), p_u8(5)],
                sky_top_rgb: [p_u8(9), p_u8(10), p_u8(11)],
                sky_bottom_rgb: [p_u8(12), p_u8(13), p_u8(14)],
                sun_core_rgb: [p_u8(15), p_u8(16), p_u8(17)],
                sun_corona_rgb: [p_u8(18), p_u8(19), p_u8(20)],
                far_clip: p_f32(25),
                fog_start: p_f32(26),
                water_rgba: [
                    p_f32(36) / 255.0,
                    p_f32(37) / 255.0,
                    p_f32(38) / 255.0,
                    p_f32(39) / 255.0,
                ],
            });
        }
        Self { entries }
    }
}

/// Vehicle RGB color palette from `data/carcols.dat`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarColsDat {
    pub palette: Vec<[u8; 3]>,
    pub vehicle_pairs: HashMap<String, Vec<(usize, usize)>>,
}

impl CarColsDat {
    pub fn parse(text: &str) -> Self {
        let mut out = Self::default();
        let mut mode = "";
        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.eq_ignore_ascii_case("end") {
                mode = "";
                continue;
            }
            if mode.is_empty() {
                if line.eq_ignore_ascii_case("col") {
                    mode = "col";
                } else if line.eq_ignore_ascii_case("car") || line.eq_ignore_ascii_case("car4") {
                    mode = "car";
                }
                continue;
            }
            match mode {
                "col" => {
                    let nums: Vec<u8> = line
                        .split([',', ' ', '\t'])
                        .filter(|s| !s.is_empty())
                        .take(3)
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if nums.len() == 3 {
                        out.palette.push([nums[0], nums[1], nums[2]]);
                    }
                }
                "car" => {
                    let parts: Vec<&str> = line
                        .split([',', ' ', '\t'])
                        .filter(|s| !s.is_empty())
                        .collect();
                    if parts.len() >= 3 {
                        let name = parts[0].to_ascii_lowercase();
                        let mut pairs = Vec::new();
                        let mut i = 1;
                        while i + 1 < parts.len() {
                            if let (Ok(c1), Ok(c2)) =
                                (parts[i].parse::<usize>(), parts[i + 1].parse::<usize>())
                            {
                                pairs.push((c1, c2));
                            }
                            i += 2;
                        }
                        out.vehicle_pairs.insert(name, pairs);
                    }
                }
                _ => {}
            }
        }
        out
    }
}

/// Weapon parameters from `data/weapon.dat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEntry {
    pub name: String,
    pub fire_type: String,
    pub target_range: f32,
    pub weapon_range: f32,
    pub model_id: i32,
    pub slot: u32,
    pub clip_size: u32,
    pub damage: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeaponDat {
    pub weapons: HashMap<String, WeaponEntry>,
}

impl WeaponDat {
    pub fn parse(text: &str) -> Self {
        let mut weapons = HashMap::new();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("ENDWEAPONDATA") {
                continue;
            }
            if !line.starts_with('$') {
                continue;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 10 {
                let name = cols[1].to_ascii_uppercase();
                weapons.insert(
                    name.clone(),
                    WeaponEntry {
                        name,
                        fire_type: cols[2].to_string(),
                        target_range: cols[3].parse().unwrap_or(20.0),
                        weapon_range: cols[4].parse().unwrap_or(35.0),
                        model_id: cols[5].parse().unwrap_or(-1),
                        slot: cols[7].parse().unwrap_or(0),
                        clip_size: cols.get(10).and_then(|s| s.parse().ok()).unwrap_or(1),
                        damage: cols.get(11).and_then(|s| s.parse().ok()).unwrap_or(20),
                    },
                );
            }
        }
        Self { weapons }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gta_dat_and_handling() {
        let dat = GtaDat::parse(
            "IMG DATA\\PATHS\\CARREC.IMG\nIDE DATA\\MAPS\\LA\\LAn.IDE\nIPL DATA\\MAPS\\LA\\LAn.IPL\n",
        );
        assert_eq!(dat.img_archives.len(), 1);
        assert_eq!(dat.ide_files.len(), 1);
        assert_eq!(dat.ipl_files.len(), 1);

        let handling_line = "BUFFALO 1500.0 4000.0 2.0 0.0 0.0 -0.1 85 0.7 0.9 0.5 5 200.0 28.0 5.0 R P 11.0 0.45 0 30.0 1.2 0.12 0.0 0.28 -0.24 0.5 0.4 0.25 0.50 35000 2800 10200000 1 1 0";
        let cfg = HandlingCfg::parse(handling_line);
        let buffalo = cfg.vehicles.get("BUFFALO").unwrap();
        assert_eq!(buffalo.mass, 1500.0);
        assert_eq!(buffalo.drive_type, 'R');
        assert_eq!(buffalo.max_velocity, 200.0);
    }
}
