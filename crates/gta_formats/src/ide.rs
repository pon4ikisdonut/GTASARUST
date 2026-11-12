use serde::{Deserialize, Serialize};

/// Item definition bitflags (`eItemDefinitionFlags`) from GTA: San Andreas `.ide` files.
pub const IDE_FLAG_IS_ROAD: u32 = 0x0000_0001;
pub const IDE_FLAG_DRAW_LAST: u32 = 0x0000_0004;
pub const IDE_FLAG_ADDITIVE: u32 = 0x0000_0008;
pub const IDE_FLAG_NO_ZWRITE: u32 = 0x0000_0040;
pub const IDE_FLAG_DONT_RECEIVE_SHADOWS: u32 = 0x0000_0080;
pub const IDE_FLAG_IS_GLASS_TYPE_1: u32 = 0x0000_0200;
pub const IDE_FLAG_IS_GLASS_TYPE_2: u32 = 0x0000_0400;
pub const IDE_FLAG_IS_GARAGE_DOOR: u32 = 0x0000_0800;
pub const IDE_FLAG_IS_DAMAGEABLE: u32 = 0x0000_1000;
pub const IDE_FLAG_IS_TREE: u32 = 0x0000_2000;
pub const IDE_FLAG_IS_PALM: u32 = 0x0000_4000;
pub const IDE_FLAG_DISABLE_BACKFACE_CULLING: u32 = 0x0020_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeObjectDef {
    pub id: u32,
    pub model_name: String,
    pub txd_name: String,
    pub draw_distance: f32,
    pub flags: u32,
    pub time_on: Option<u8>,
    pub time_off: Option<u8>,
    pub anim_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeVehicleDef {
    pub id: u32,
    pub model_name: String,
    pub txd_name: String,
    pub vehicle_type: String,
    pub handling_id: String,
    pub game_name: String,
    pub anims: String,
    pub class: String,
    pub frequency: u32,
    pub flags: u32,
    pub comp_rules: u32,
    pub wheel_model_id: i32,
    pub wheel_scale_front: f32,
    pub wheel_scale_rear: f32,
    pub wheel_upgrade_class: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdePedDef {
    pub id: u32,
    pub model_name: String,
    pub txd_name: String,
    pub ped_type: String,
    pub stat_type: String,
    pub anim_group: String,
    pub cars_mask: u32,
    pub flags: u32,
    pub anim_file: String,
    pub radio1: i32,
    pub radio2: i32,
    pub voice_type: String,
    pub voice1: String,
    pub voice2: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeTxdParent {
    pub child_txd: String,
    pub parent_txd: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdeFile {
    pub objects: Vec<IdeObjectDef>,
    pub weapons: Vec<IdeObjectDef>,
    pub vehicles: Vec<IdeVehicleDef>,
    pub peds: Vec<IdePedDef>,
    pub txd_parents: Vec<IdeTxdParent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdeSection {
    None,
    Objs,
    Tobj,
    Weap,
    Hier,
    Anim,
    Cars,
    Peds,
    Txdp,
    Other,
}

impl IdeFile {
    /// Parse a GTA: San Andreas `.ide` definition file.
    pub fn parse(text: &str) -> Self {
        let mut out = Self::default();
        let mut section = IdeSection::None;

        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower == "end" {
                section = IdeSection::None;
                continue;
            }
            if section == IdeSection::None {
                section = match lower.as_str() {
                    "objs" => IdeSection::Objs,
                    "tobj" => IdeSection::Tobj,
                    "weap" => IdeSection::Weap,
                    "hier" => IdeSection::Hier,
                    "anim" => IdeSection::Anim,
                    "cars" => IdeSection::Cars,
                    "peds" => IdeSection::Peds,
                    "txdp" => IdeSection::Txdp,
                    _ => IdeSection::Other,
                };
                continue;
            }

            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            match section {
                IdeSection::Objs if fields.len() >= 5 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        // Format: ID, Model, Txd, [MeshCount,] DrawDist[, Flags]
                        let (draw_distance, flags) = if fields.len() >= 6 {
                            if let Ok(dd) = fields[4].parse::<f32>() {
                                (dd, fields[5].parse::<u32>().unwrap_or(0))
                            } else {
                                (
                                    fields[3].parse::<f32>().unwrap_or(100.0),
                                    fields[4].parse::<u32>().unwrap_or(0),
                                )
                            }
                        } else {
                            (
                                fields[3].parse::<f32>().unwrap_or(100.0),
                                fields[4].parse::<u32>().unwrap_or(0),
                            )
                        };
                        out.objects.push(IdeObjectDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            draw_distance,
                            flags,
                            time_on: None,
                            time_off: None,
                            anim_name: None,
                        });
                    }
                }
                IdeSection::Tobj if fields.len() >= 7 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.objects.push(IdeObjectDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            draw_distance: fields[3].parse::<f32>().unwrap_or(100.0),
                            flags: fields[4].parse::<u32>().unwrap_or(0),
                            time_on: fields[5].parse::<u8>().ok(),
                            time_off: fields[6].parse::<u8>().ok(),
                            anim_name: None,
                        });
                    }
                }
                IdeSection::Anim if fields.len() >= 6 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.objects.push(IdeObjectDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            draw_distance: fields[4].parse::<f32>().unwrap_or(100.0),
                            flags: fields[5].parse::<u32>().unwrap_or(0),
                            time_on: None,
                            time_off: None,
                            anim_name: Some(fields[3].to_string()),
                        });
                    }
                }
                IdeSection::Weap if fields.len() >= 7 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.weapons.push(IdeObjectDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            draw_distance: fields[5].parse::<f32>().unwrap_or(50.0),
                            flags: fields[6].parse::<u32>().unwrap_or(0),
                            time_on: None,
                            time_off: None,
                            anim_name: Some(fields[3].to_string()),
                        });
                    }
                }
                IdeSection::Hier if fields.len() >= 3 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.objects.push(IdeObjectDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            draw_distance: 300.0,
                            flags: 0,
                            time_on: None,
                            time_off: None,
                            anim_name: None,
                        });
                    }
                }
                IdeSection::Cars if fields.len() >= 15 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.vehicles.push(IdeVehicleDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            vehicle_type: fields[3].to_string(),
                            handling_id: fields[4].to_string(),
                            game_name: fields[5].to_string(),
                            anims: fields[6].to_string(),
                            class: fields[7].to_string(),
                            frequency: fields[8].parse().unwrap_or(10),
                            flags: fields[9].parse().unwrap_or(0),
                            comp_rules: u32::from_str_radix(fields[10], 16).unwrap_or(0),
                            wheel_model_id: fields[11].parse().unwrap_or(-1),
                            wheel_scale_front: fields[12].parse().unwrap_or(0.7),
                            wheel_scale_rear: fields[13].parse().unwrap_or(0.7),
                            wheel_upgrade_class: fields[14].parse().unwrap_or(-1),
                        });
                    }
                }
                IdeSection::Peds if fields.len() >= 14 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.peds.push(IdePedDef {
                            id,
                            model_name: fields[1].to_string(),
                            txd_name: fields[2].to_string(),
                            ped_type: fields[3].to_string(),
                            stat_type: fields[4].to_string(),
                            anim_group: fields[5].to_string(),
                            cars_mask: u32::from_str_radix(fields[6], 16).unwrap_or(0),
                            flags: fields[7].parse().unwrap_or(0),
                            anim_file: fields[8].to_string(),
                            radio1: fields[9].parse().unwrap_or(0),
                            radio2: fields[10].parse().unwrap_or(0),
                            voice_type: fields[11].to_string(),
                            voice1: fields[12].to_string(),
                            voice2: fields[13].to_string(),
                        });
                    }
                }
                IdeSection::Txdp if fields.len() >= 2 => {
                    out.txd_parents.push(IdeTxdParent {
                        child_txd: fields[0].to_string(),
                        parent_txd: fields[1].to_string(),
                    });
                }
                _ => {}
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ide_sections() {
        let sample = "\
objs
320, airtrain_vlo, generic, 1, 2000, 0
5390, laeskateparkLA, glenpark7_lae, 150, 0
end
cars
411, infernus, infernus, car, INFERNUS, INFERNU, null, executive, 5, 0, 0, -1, 0.7, 0.7, 0
end
";
        let ide = IdeFile::parse(sample);
        assert_eq!(ide.objects.len(), 2);
        assert_eq!(ide.objects[0].draw_distance, 2000.0);
        assert_eq!(ide.objects[1].draw_distance, 150.0);
        assert_eq!(ide.vehicles.len(), 1);
        assert_eq!(ide.vehicles[0].model_name, "infernus");
    }
}
