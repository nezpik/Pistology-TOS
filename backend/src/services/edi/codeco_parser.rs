use crate::models::edi::{ParsedCodecoData, ParsedCodecoMovement};

/// Parse a CODECO (Container gate-in/gate-out) EDI message
/// CODECO messages contain container gate movement information
pub fn parse_codeco(content: &str) -> Result<ParsedCodecoData, String> {
    let mut gate = None;
    let mut movements = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_container_id = None;
    let mut current_movement_type = None;
    let mut current_truck_plate = None;
    let mut current_iso_type = None;

    for line in lines {
        let parts: Vec<&str> = line.split('+').collect();
        if parts.is_empty() {
            continue;
        }

        let segment = parts[0];

        match segment {
            // LOC segment - Gate location
            "LOC" => {
                if parts.len() > 2 {
                    let qualifier = parts.get(1).unwrap_or(&"");
                    if *qualifier == "9" || *qualifier == "11" {
                        // Gate location
                        if let Some(location) = parts.get(2) {
                            let loc_parts: Vec<&str> = location.split(':').collect();
                            if let Some(g) = loc_parts.first() {
                                if gate.is_none() {
                                    gate = Some(g.to_string());
                                }
                            }
                        }
                    }
                }
            }
            // EQD segment - Equipment details (container)
            "EQD" => {
                // Save previous movement if exists
                if let Some(cid) = current_container_id.take() {
                    movements.push(ParsedCodecoMovement {
                        container_id: cid,
                        movement_type: current_movement_type.take(),
                        truck_license_plate: current_truck_plate.take(),
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
            // TDT segment - Transport details (truck)
            "TDT" => {
                if parts.len() > 8 {
                    // TDT+1 or TDT+3 for road transport
                    let transport_stage = parts.get(1).unwrap_or(&"");
                    if *transport_stage == "1" || *transport_stage == "3" {
                        // Look for truck license plate in transport means
                        if let Some(means) = parts.get(8) {
                            let means_parts: Vec<&str> = means.split(':').collect();
                            if let Some(plate) = means_parts.first() {
                                current_truck_plate = Some(plate.to_string());
                            }
                        }
                    }
                }
            }
            // RFF segment - Reference (may contain truck plate)
            "RFF" => {
                if parts.len() > 1 {
                    if let Some(ref_data) = parts.get(1) {
                        let ref_parts: Vec<&str> = ref_data.split(':').collect();
                        if let Some(ref_qualifier) = ref_parts.first() {
                            match *ref_qualifier {
                                "CN" | "TN" => {
                                    // Truck/transport reference
                                    if let Some(ref_value) = ref_parts.get(1) {
                                        if current_truck_plate.is_none() {
                                            current_truck_plate = Some(ref_value.to_string());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            // HAN segment - Handling instructions (movement type)
            "HAN" => {
                if parts.len() > 1 {
                    if let Some(handling) = parts.get(1) {
                        let han_parts: Vec<&str> = handling.split(':').collect();
                        if let Some(han_code) = han_parts.first() {
                            // Common codes: 2 = load/in, 3 = discharge/out
                            match *han_code {
                                "2" | "5" => {
                                    current_movement_type = Some("IN".to_string());
                                }
                                "3" | "6" => {
                                    current_movement_type = Some("OUT".to_string());
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
                            if upper.contains("GATE-IN") || upper.contains("GATE IN") {
                                current_movement_type = Some("IN".to_string());
                            } else if upper.contains("GATE-OUT") || upper.contains("GATE OUT") {
                                current_movement_type = Some("OUT".to_string());
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
        movements.push(ParsedCodecoMovement {
            container_id: cid,
            movement_type: current_movement_type,
            truck_license_plate: current_truck_plate,
            iso_container_type: current_iso_type,
        });
    }

    Ok(ParsedCodecoData {
        gate,
        movements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codeco() {
        let sample = r#"UNH+1+CODECO:D:95B:UN
LOC+9+GATE1:139:6
EQD+CN+CONTAINER123+45G1:102
TDT+3++1+++++ABC123:::5
HAN+2
UNT+5+1"#;

        let result = parse_codeco(sample);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.gate, Some("GATE1".to_string()));
        assert_eq!(data.movements.len(), 1);
    }
}
