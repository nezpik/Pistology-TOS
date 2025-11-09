use crate::models::edi::{ParsedCoarriData, ParsedCoarriMovement};

/// Parse a COARRI (Container discharge/loading order) EDI message
/// COARRI messages contain container movement information for vessel operations
pub fn parse_coarri(content: &str) -> Result<ParsedCoarriData, String> {
    let mut vessel_name = None;
    let mut voyage_number = None;
    let mut movements = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_container_id = None;
    let mut current_movement_type = None;
    let mut current_stowage = None;
    let mut current_iso_type = None;

    for line in lines {
        let parts: Vec<&str> = line.split('+').collect();
        if parts.is_empty() {
            continue;
        }

        let segment = parts[0];

        match segment {
            // TDT segment - Transport details (vessel information)
            "TDT" => {
                if parts.len() > 8 {
                    if let Some(voyage) = parts.get(2) {
                        if !voyage.is_empty() {
                            voyage_number = Some(voyage.to_string());
                        }
                    }
                    if let Some(vessel) = parts.get(8) {
                        if !vessel.is_empty() {
                            let vessel_parts: Vec<&str> = vessel.split(':').collect();
                            if let Some(v) = vessel_parts.first() {
                                vessel_name = Some(v.to_string());
                            }
                        }
                    }
                }
            }
            // EQD segment - Equipment details (container)
            "EQD" => {
                // Save previous movement if exists
                if let Some(cid) = current_container_id.take() {
                    movements.push(ParsedCoarriMovement {
                        container_id: cid,
                        movement_type: current_movement_type.take(),
                        stowage_location: current_stowage.take(),
                        iso_container_type: current_iso_type.take(),
                    });
                }

                if parts.len() > 2 {
                    if let Some(container) = parts.get(2) {
                        if !container.is_empty() {
                            current_container_id = Some(container.to_string());
                        }
                    }
                    if let Some(iso_type) = parts.get(3) {
                        if !iso_type.is_empty() {
                            let iso_parts: Vec<&str> = iso_type.split(':').collect();
                            if let Some(iso) = iso_parts.first() {
                                current_iso_type = Some(iso.to_string());
                            }
                        }
                    }
                }
            }
            // RFF segment - Reference (movement type)
            "RFF" => {
                if parts.len() > 1 {
                    if let Some(ref_data) = parts.get(1) {
                        let ref_parts: Vec<&str> = ref_data.split(':').collect();
                        if let Some(ref_qualifier) = ref_parts.first() {
                            // Common qualifiers: BM (Bill of Lading), ABO (Movement reference)
                            match *ref_qualifier {
                                "BM" | "ABO" => {
                                    if let Some(ref_value) = ref_parts.get(1) {
                                        // Try to determine movement type from reference
                                        let upper = ref_value.to_uppercase();
                                        if upper.contains("LOAD") {
                                            current_movement_type = Some("LOAD".to_string());
                                        } else if upper.contains("DISCH") {
                                            current_movement_type = Some("DISCHARGE".to_string());
                                        } else if upper.contains("SHIFT") {
                                            current_movement_type = Some("SHIFT".to_string());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            // FTX segment - Free text (may contain movement info)
            "FTX" => {
                if parts.len() > 3 {
                    if let Some(text) = parts.get(3) {
                        let upper = text.to_uppercase();
                        if current_movement_type.is_none() {
                            if upper.contains("LOAD") {
                                current_movement_type = Some("LOAD".to_string());
                            } else if upper.contains("DISCH") {
                                current_movement_type = Some("DISCHARGE".to_string());
                            } else if upper.contains("SHIFT") {
                                current_movement_type = Some("SHIFT".to_string());
                            }
                        }
                    }
                }
            }
            // LOC segment - Stowage location
            "LOC" => {
                if parts.len() > 2 {
                    let qualifier = parts.get(1).unwrap_or(&"");
                    if *qualifier == "147" || *qualifier == "7" {
                        // Stowage location
                        if let Some(location) = parts.get(2) {
                            let loc_parts: Vec<&str> = location.split(':').collect();
                            if let Some(stow) = loc_parts.first() {
                                current_stowage = Some(stow.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Save last movement if exists
    if let Some(cid) = current_container_id {
        movements.push(ParsedCoarriMovement {
            container_id: cid,
            movement_type: current_movement_type,
            stowage_location: current_stowage,
            iso_container_type: current_iso_type,
        });
    }

    Ok(ParsedCoarriData {
        vessel_name,
        voyage_number,
        movements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coarri() {
        let sample = r#"UNH+1+COARRI:D:95B:UN
TDT+20+V123+1++CARRIER+++VESSEL_NAME:::5
EQD+CN+CONTAINER123+45G1:102
RFF+BM:LOAD123
LOC+147+010203
UNT+5+1"#;

        let result = parse_coarri(sample);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.vessel_name, Some("VESSEL_NAME".to_string()));
        assert_eq!(data.movements.len(), 1);
    }
}
