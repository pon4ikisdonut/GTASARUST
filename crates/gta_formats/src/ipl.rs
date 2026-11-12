use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IplInstance {
    pub id: u32,
    pub model_name: String,
    pub interior: i32,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // x, y, z, w quaternion
    pub lod: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IplCarSpawn {
    pub position: [f32; 3],
    pub angle: f32,
    pub car_id: i32,
    pub color1: i32,
    pub color2: i32,
    pub force_spawn: i32,
    pub alarm: i32,
    pub door_lock: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IplEntryExit {
    pub entrance: [f32; 3],
    pub entrance_rot: f32,
    pub size: [f32; 3],
    pub exit: [f32; 3],
    pub exit_rot: f32,
    pub target_interior: i32,
    pub flags: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IplZoneBox {
    pub name: String,
    pub zone_type: i32,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub level: i32,
    pub text_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IplOcclusionBox {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IplFile {
    pub instances: Vec<IplInstance>,
    pub car_spawns: Vec<IplCarSpawn>,
    pub entry_exits: Vec<IplEntryExit>,
    pub zones: Vec<IplZoneBox>,
    pub occluders: Vec<IplOcclusionBox>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IplSection {
    None,
    Inst,
    Cars,
    Enex,
    Zone,
    Occl,
    Other,
}

impl IplFile {
    /// Parse either a text `.ipl` file or a binary `bnry` `.ipl` stream from `gta3.img`.
    pub fn parse_bytes(data: &[u8]) -> Result<Self, FormatError> {
        if data.starts_with(b"bnry") {
            Self::parse_binary(data)
        } else {
            let text = String::from_utf8_lossy(data);
            Ok(Self::parse_text(&text))
        }
    }

    /// Parse a binary (`bnry`) streaming IPL file stored inside `gta3.img`.
    pub fn parse_binary(data: &[u8]) -> Result<Self, FormatError> {
        if !data.starts_with(b"bnry") {
            return Err(FormatError::InvalidMagic {
                expected: "bnry".to_string(),
                actual: String::from_utf8_lossy(data.get(0..4).unwrap_or_default()).into_owned(),
            });
        }
        let mut r = BinReader::new(data);
        r.skip(4)?; // "bnry"
        let num_instances = r.read_u32_le()? as usize;
        let _unk1 = r.read_u32_le()?;
        let _unk2 = r.read_u32_le()?;
        let _unk3 = r.read_u32_le()?;
        let num_cars = r.read_u32_le()? as usize;
        let _unk4 = r.read_u32_le()?;
        let instances_offset = r.read_u32_le()? as usize;
        r.skip(28)?; // unused stream header fields up to 0x3C
        let cars_offset = r.read_u32_le()? as usize;

        let mut out = Self::default();
        if num_instances > 0 {
            r.seek(instances_offset)?;
            out.instances.reserve(num_instances.min(100_000));
            for _ in 0..num_instances {
                if r.remaining() < 40 {
                    break;
                }
                let position = r.read_vec3_le()?;
                let rx = r.read_f32_le()?;
                let ry = r.read_f32_le()?;
                let rz = r.read_f32_le()?;
                let rw = r.read_f32_le()?;
                let id = r.read_u32_le()?;
                let interior = r.read_i32_le()?;
                let lod = r.read_i32_le()?;
                out.instances.push(IplInstance {
                    id,
                    model_name: String::new(), // resolved by model ID via IDE table
                    interior,
                    position,
                    rotation: [rx, ry, rz, rw],
                    lod,
                });
            }
        }

        if num_cars > 0 && cars_offset < data.len() {
            r.seek(cars_offset)?;
            out.car_spawns.reserve(num_cars.min(50_000));
            for _ in 0..num_cars {
                if r.remaining() < 48 {
                    break;
                }
                let position = r.read_vec3_le()?;
                let angle = r.read_f32_le()?;
                let car_id = r.read_i32_le()?;
                let color1 = r.read_i32_le()?;
                let color2 = r.read_i32_le()?;
                let force_spawn = r.read_i32_le()?;
                let alarm = r.read_i32_le()?;
                let door_lock = r.read_i32_le()?;
                let _min_delay = r.read_i32_le()?;
                let _max_delay = r.read_i32_le()?;
                out.car_spawns.push(IplCarSpawn {
                    position,
                    angle,
                    car_id,
                    color1,
                    color2,
                    force_spawn,
                    alarm,
                    door_lock,
                });
            }
        }

        Ok(out)
    }

    /// Parse a text `.ipl` or `.zon` file.
    pub fn parse_text(text: &str) -> Self {
        let mut out = Self::default();
        let mut section = IplSection::None;

        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let lower = line.to_ascii_lowercase();
            if lower == "end" {
                section = IplSection::None;
                continue;
            }
            if section == IplSection::None {
                section = match lower.as_str() {
                    "inst" => IplSection::Inst,
                    "cars" => IplSection::Cars,
                    "enex" => IplSection::Enex,
                    "zone" => IplSection::Zone,
                    "occl" => IplSection::Occl,
                    _ => IplSection::Other,
                };
                continue;
            }

            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            match section {
                IplSection::Inst if fields.len() >= 11 => {
                    if let Ok(id) = fields[0].parse::<u32>() {
                        out.instances.push(IplInstance {
                            id,
                            model_name: fields[1].to_string(),
                            interior: fields[2].parse().unwrap_or(0),
                            position: [
                                fields[3].parse().unwrap_or(0.0),
                                fields[4].parse().unwrap_or(0.0),
                                fields[5].parse().unwrap_or(0.0),
                            ],
                            rotation: [
                                fields[6].parse().unwrap_or(0.0),
                                fields[7].parse().unwrap_or(0.0),
                                fields[8].parse().unwrap_or(0.0),
                                fields[9].parse().unwrap_or(1.0),
                            ],
                            lod: fields[10].parse().unwrap_or(-1),
                        });
                    }
                }
                IplSection::Cars if fields.len() >= 10 => {
                    out.car_spawns.push(IplCarSpawn {
                        position: [
                            fields[0].parse().unwrap_or(0.0),
                            fields[1].parse().unwrap_or(0.0),
                            fields[2].parse().unwrap_or(0.0),
                        ],
                        angle: fields[3].parse().unwrap_or(0.0),
                        car_id: fields[4].parse().unwrap_or(-1),
                        color1: fields[5].parse().unwrap_or(-1),
                        color2: fields[6].parse().unwrap_or(-1),
                        force_spawn: fields[7].parse().unwrap_or(0),
                        alarm: fields[8].parse().unwrap_or(0),
                        door_lock: fields[9].parse().unwrap_or(0),
                    });
                }
                IplSection::Enex if fields.len() >= 14 => {
                    out.entry_exits.push(IplEntryExit {
                        entrance: [
                            fields[0].parse().unwrap_or(0.0),
                            fields[1].parse().unwrap_or(0.0),
                            fields[2].parse().unwrap_or(0.0),
                        ],
                        entrance_rot: fields[3].parse().unwrap_or(0.0),
                        size: [
                            fields[4].parse().unwrap_or(1.0),
                            fields[5].parse().unwrap_or(1.0),
                            fields[6].parse().unwrap_or(1.0),
                        ],
                        exit: [
                            fields[7].parse().unwrap_or(0.0),
                            fields[8].parse().unwrap_or(0.0),
                            fields[9].parse().unwrap_or(0.0),
                        ],
                        exit_rot: fields[10].parse().unwrap_or(0.0),
                        target_interior: fields[11].parse().unwrap_or(0),
                        flags: fields[12].parse().unwrap_or(0),
                        name: fields[13].trim_matches('"').to_string(),
                    });
                }
                IplSection::Zone if fields.len() >= 10 => {
                    out.zones.push(IplZoneBox {
                        name: fields[0].to_string(),
                        zone_type: fields[1].parse().unwrap_or(0),
                        min: [
                            fields[2].parse().unwrap_or(0.0),
                            fields[3].parse().unwrap_or(0.0),
                            fields[4].parse().unwrap_or(0.0),
                        ],
                        max: [
                            fields[5].parse().unwrap_or(0.0),
                            fields[6].parse().unwrap_or(0.0),
                            fields[7].parse().unwrap_or(0.0),
                        ],
                        level: fields[8].parse().unwrap_or(0),
                        text_label: fields[9].to_string(),
                    });
                }
                IplSection::Occl if fields.len() >= 7 => {
                    out.occluders.push(IplOcclusionBox {
                        center: [
                            fields[0].parse().unwrap_or(0.0),
                            fields[1].parse().unwrap_or(0.0),
                            fields[2].parse().unwrap_or(0.0),
                        ],
                        size: [
                            fields[3].parse().unwrap_or(0.0),
                            fields[4].parse().unwrap_or(0.0),
                            fields[5].parse().unwrap_or(0.0),
                        ],
                        rotation: [
                            fields[6].parse().unwrap_or(0.0),
                            fields.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            fields.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        ],
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
    fn parses_text_and_binary_ipl() {
        let text = "inst\n3563, LODpmedhos2_LAe, 0, 2063.1, -1731.7, 15.2, 0, 0, 0.707, 0.707, -1\nend\nenex\n2495.33, -1690.75, 13.7847, 0, 1.4, 1.4, 8, 2495.33, -1690.25, 13.7847, 0, 0, 4, \"CARLS\", 0, 2, 0, 24\nend\n";
        let parsed = IplFile::parse_text(text);
        assert_eq!(parsed.instances.len(), 1);
        assert_eq!(parsed.instances[0].id, 3563);
        assert_eq!(parsed.entry_exits.len(), 1);
        assert_eq!(parsed.entry_exits[0].name, "CARLS");
        assert_eq!(parsed.entry_exits[0].target_interior, 0);
        assert!((parsed.entry_exits[0].exit[0] - 2495.33).abs() < 0.01);
        assert!((parsed.entry_exits[0].exit[1] - (-1690.25)).abs() < 0.01);

        // Synthetic binary IPL with 1 instance at offset 0x4C (76) and 1 car spawn at 0x3C (116)
        let mut bin = vec![0u8; 76 + 40 + 48];
        bin[0..4].copy_from_slice(b"bnry");
        bin[4..8].copy_from_slice(&1u32.to_le_bytes()); // 1 inst
        bin[20..24].copy_from_slice(&1u32.to_le_bytes()); // 1 car spawn
        bin[28..32].copy_from_slice(&76u32.to_le_bytes()); // inst offset
        bin[60..64].copy_from_slice(&116u32.to_le_bytes()); // cars offset at 0x3C
        bin[76..80].copy_from_slice(&100.0f32.to_le_bytes());
        bin[104..108].copy_from_slice(&5390u32.to_le_bytes());
        bin[108..112].copy_from_slice(&0i32.to_le_bytes());
        bin[112..116].copy_from_slice(&(-1i32).to_le_bytes());
        // Car record at offset 116: pos=(2490.0, -1670.0, 13.3), angle=1.57, car_id=492
        bin[116..120].copy_from_slice(&2490.0f32.to_le_bytes());
        bin[120..124].copy_from_slice(&(-1670.0f32).to_le_bytes());
        bin[124..128].copy_from_slice(&13.3f32.to_le_bytes());
        bin[128..132].copy_from_slice(&1.57f32.to_le_bytes());
        bin[132..136].copy_from_slice(&492i32.to_le_bytes());
        let bparsed = IplFile::parse_bytes(&bin).unwrap();
        assert_eq!(bparsed.instances.len(), 1);
        assert_eq!(bparsed.instances[0].id, 5390);
        assert!((bparsed.instances[0].position[0] - 100.0).abs() < 0.01);
        assert_eq!(bparsed.car_spawns.len(), 1);
        assert_eq!(bparsed.car_spawns[0].car_id, 492);
        assert!((bparsed.car_spawns[0].position[2] - 13.3).abs() < 0.01);
    }
}
