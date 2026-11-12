use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

/// Single vehicle, boat, or pedestrian waypoint node parsed from a `NODES*.DAT` file inside `gta3.img`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub position: [f32; 3],
    pub link_start: u16,
    pub area_id: u16,
    pub node_id: u16,
    pub link_count: u8,
    pub width: f32,
    pub node_type: u8,
    pub is_ped: bool,
    pub is_water: bool,
    pub is_highway: bool,
    pub flags: u32,
}

/// Directed adjacency link from one `PathNode` to `(target_area_id, target_node_id)`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PathLink {
    pub target_area_id: u16,
    pub target_node_id: u16,
}

/// Parsed San Andreas path sector (`NODES0.DAT`..`NODES63.DAT`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathsFile {
    pub num_vehicle_nodes: u32,
    pub num_ped_nodes: u32,
    pub nodes: Vec<PathNode>,
    pub links: Vec<PathLink>,
}

impl PathsFile {
    /// Parse a binary `NODES*.DAT` buffer with strict bounds checking.
    pub fn parse_bytes(data: &[u8]) -> Result<Self, FormatError> {
        let mut r = BinReader::new(data);
        let num_nodes = r.read_u32_le()? as usize;
        let num_vehicle_nodes = r.read_u32_le()?;
        let num_ped_nodes = r.read_u32_le()?;
        let num_navi_nodes = r.read_u32_le()? as usize;
        let num_links = r.read_u32_le()? as usize;

        let nodes_bytes = num_nodes
            .checked_mul(28)
            .ok_or_else(|| FormatError::Malformed {
                context: "NODES nodes size".to_string(),
                detail: "overflow".to_string(),
            })?;
        let navi_bytes = num_navi_nodes
            .checked_mul(14)
            .ok_or_else(|| FormatError::Malformed {
                context: "NODES navi size".to_string(),
                detail: "overflow".to_string(),
            })?;
        let links_bytes = num_links
            .checked_mul(4)
            .ok_or_else(|| FormatError::Malformed {
                context: "NODES links size".to_string(),
                detail: "overflow".to_string(),
            })?;
        let links_offset = 20usize
            .checked_add(nodes_bytes)
            .and_then(|v| v.checked_add(navi_bytes))
            .ok_or_else(|| FormatError::Malformed {
                context: "NODES offset".to_string(),
                detail: "overflow".to_string(),
            })?;
        let total_needed =
            links_offset
                .checked_add(links_bytes)
                .ok_or_else(|| FormatError::Malformed {
                    context: "NODES total size".to_string(),
                    detail: "overflow".to_string(),
                })?;
        if total_needed > data.len() {
            return Err(FormatError::UnexpectedEof {
                offset: links_offset,
                needed: total_needed,
                remaining: data.len(),
            });
        }

        let mut nodes = Vec::with_capacity(num_nodes);
        for idx in 0..num_nodes {
            r.skip(8)?; // memory_addr + unknown
            let px = f32::from(r.read_i16_le()?) / 8.0;
            let py = f32::from(r.read_i16_le()?) / 8.0;
            let pz = f32::from(r.read_i16_le()?) / 8.0;
            let _heuristic = r.read_i16_le()?;
            let link_start = r.read_u16_le()?;
            let area_id = r.read_u16_le()?;
            let node_id = r.read_u16_le()?;
            let path_width = r.read_u8()?;
            let node_type = r.read_u8()?;
            let flags = r.read_u32_le()?;

            nodes.push(PathNode {
                position: [px, py, pz],
                link_start,
                area_id,
                node_id,
                link_count: (flags & 0x0F) as u8,
                width: f32::from(path_width) / 8.0,
                node_type,
                is_ped: idx >= num_vehicle_nodes as usize,
                is_water: (flags & (1 << 7)) != 0,
                is_highway: (flags & (1 << 12)) != 0,
                flags,
            });
        }

        r.seek(links_offset)?;
        let mut links = Vec::with_capacity(num_links);
        for _ in 0..num_links {
            let target_area_id = r.read_u16_le()?;
            let target_node_id = r.read_u16_le()?;
            links.push(PathLink {
                target_area_id,
                target_node_id,
            });
        }

        Ok(Self {
            num_vehicle_nodes,
            num_ped_nodes,
            nodes,
            links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_nodes_dat() {
        // 2 nodes (1 veh, 1 ped), 0 navi nodes, 2 links
        let mut buf = Vec::new();
        buf.extend_from_slice(&2u32.to_le_bytes()); // num_nodes
        buf.extend_from_slice(&1u32.to_le_bytes()); // num_vehicle_nodes
        buf.extend_from_slice(&1u32.to_le_bytes()); // num_ped_nodes
        buf.extend_from_slice(&0u32.to_le_bytes()); // num_navi_nodes
        buf.extend_from_slice(&2u32.to_le_bytes()); // num_links

        // Node 0 (vehicle road node at (2488.0, -1664.0, 13.0))
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&(2488i16 * 8).to_le_bytes());
        buf.extend_from_slice(&(-1664i16 * 8).to_le_bytes());
        buf.extend_from_slice(&(13i16 * 8).to_le_bytes());
        buf.extend_from_slice(&0i16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // link_start = 0
        buf.extend_from_slice(&15u16.to_le_bytes()); // area_id = 15
        buf.extend_from_slice(&0u16.to_le_bytes()); // node_id = 0
        buf.push(16); // width = 2.0
        buf.push(1); // node_type = 1
        buf.extend_from_slice(&0x000f_1001u32.to_le_bytes()); // 1 link

        // Node 1 (ped node at (2496.0, -1664.0, 13.25))
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&(2496i16 * 8).to_le_bytes());
        buf.extend_from_slice(&(-1664i16 * 8).to_le_bytes());
        buf.extend_from_slice(&106i16.to_le_bytes()); // 13.25
        buf.extend_from_slice(&0i16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // link_start = 1
        buf.extend_from_slice(&15u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.push(8);
        buf.push(0);
        buf.extend_from_slice(&1u32.to_le_bytes());

        // Links: 0 -> (15, 0) [self-test target], 1 -> (15, 1)
        buf.extend_from_slice(&15u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&15u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());

        let parsed = PathsFile::parse_bytes(&buf).unwrap();
        assert_eq!(parsed.nodes.len(), 2);
        assert!(!parsed.nodes[0].is_ped);
        assert!(parsed.nodes[1].is_ped);
        assert!((parsed.nodes[0].position[0] - 2488.0).abs() < 0.01);
        assert!((parsed.nodes[1].position[2] - 13.25).abs() < 0.01);
        assert_eq!(parsed.nodes[0].link_count, 1);
        assert_eq!(parsed.links.len(), 2);
        assert_eq!(parsed.links[0].target_area_id, 15);
        assert_eq!(parsed.links[0].target_node_id, 1);
    }
}
