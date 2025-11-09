use crate::models::edi::{ParsedBaplieContainer, ParsedBaplieData};

/// Parse a BAPLIE (Bayplan/stowage plan occupied and empty locations) EDI message
/// BAPLIE messages contain vessel stowage information
pub fn parse_baplie(content: &str) -> Result<ParsedBaplieData, String> {
    let mut vessel_name = None;
    let mut voyage_number = None;
    let mut port_of_loading = None;
    let mut port_of_discharge = None;
    let mut containers = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_container_id = None;
    let mut current_bay = None;
    let mut current_row = None;
    let mut current_tier = None;
    let mut current_size = None;
    let mut current_type = None;
    let mut current_weight = None;

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
                    // TDT+20+[voyage]+1++[carrier]+++[vessel]
                    if let Some(voyage) = parts.get(2) {
                        if !voyage.is_empty() {
                            voyage_number = Some(voyage.to_string());
                        }
                    }
                    if let Some(vessel) = parts.get(8) {
                        if !vessel.is_empty() {
                            // Extract vessel name from code (may have ::: delimiters)
                            let vessel_parts: Vec<&str> = vessel.split(':').collect();
                            if let Some(v) = vessel_parts.first() {
                                vessel_name = Some(v.to_string());
                            }
                        }
                    }
                }
            }
            // LOC segment - Location information (ports)
            "LOC" => {
                if parts.len() > 2 {
                    let qualifier = parts.get(1).unwrap_or(&"");
                    let location = parts.get(2).unwrap_or(&"");

                    match *qualifier {
                        "5" => {
                            // Port of loading
                            let loc_parts: Vec<&str> = location.split(':').collect();
                            if let Some(port) = loc_parts.first() {
                                port_of_loading = Some(port.to_string());
                            }
                        }
                        "61" | "7" => {
                            // Port of discharge
                            let loc_parts: Vec<&str> = location.split(':').collect();
                            if let Some(port) = loc_parts.first() {
                                port_of_discharge = Some(port.to_string());
                            }
                        }
                        "147" => {
                            // Stowage location (bay:row:tier)
                            let stow_parts: Vec<&str> = location.split(':').collect();
                            if stow_parts.len() >= 3 {
                                current_bay = Some(stow_parts[0].to_string());
                                current_row = Some(stow_parts[1].to_string());
                                current_tier = Some(stow_parts[2].to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            // EQD segment - Equipment details (container)
            "EQD" => {
                // Save previous container if exists
                if let Some(cid) = current_container_id.take() {
                    containers.push(ParsedBaplieContainer {
                        container_id: cid,
                        bay: current_bay.take(),
                        row: current_row.take(),
                        tier: current_tier.take(),
                        size: current_size.take(),
                        container_type: current_type.take(),
                        weight: current_weight.take(),
                    });
                }

                if parts.len() > 2 {
                    // EQD+CN+[container_id]+[size/type code]
                    if let Some(container) = parts.get(2) {
                        if !container.is_empty() {
                            current_container_id = Some(container.to_string());
                        }
                    }
                    if let Some(size_type) = parts.get(3) {
                        if !size_type.is_empty() {
                            // Size/type code like "45G1" (45 ft, G1 type)
                            let st_parts: Vec<&str> = size_type.split(':').collect();
                            if let Some(st) = st_parts.first() {
                                if st.len() >= 2 {
                                    current_size = Some(st[..2].to_string());
                                    current_type = Some(st.to_string());
                                }
                            }
                        }
                    }
                }
            }
            // MEA segment - Measurements (weight)
            "MEA" => {
                if parts.len() > 3 {
                    let qualifier = parts.get(1).unwrap_or(&"");
                    if *qualifier == "AAE" || *qualifier == "VGM" {
                        // Weight measurement
                        if let Some(weight_info) = parts.get(3) {
                            let weight_parts: Vec<&str> = weight_info.split(':').collect();
                            if let Some(w) = weight_parts.last() {
                                if let Ok(weight) = w.parse::<f64>() {
                                    current_weight = Some(weight);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Save last container if exists
    if let Some(cid) = current_container_id {
        containers.push(ParsedBaplieContainer {
            container_id: cid,
            bay: current_bay,
            row: current_row,
            tier: current_tier,
            size: current_size,
            container_type: current_type,
            weight: current_weight,
        });
    }

    Ok(ParsedBaplieData {
        vessel_name,
        voyage_number,
        port_of_loading,
        port_of_discharge,
        containers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_baplie() {
        let sample = r#"UNH+1+BAPLIE:D:95B:UN
TDT+20+V123+1++CARRIER+++VESSEL_NAME:::5
LOC+5+USNYC:139:6
LOC+61+GBSOU:139:6
EQD+CN+CONTAINER123+45G1:102
LOC+147+010203:5
MEA+AAE+VGM+KGM:15000
UNT+7+1"#;

        let result = parse_baplie(sample);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.vessel_name, Some("VESSEL_NAME".to_string()));
        assert_eq!(data.voyage_number, Some("V123".to_string()));
        assert_eq!(data.containers.len(), 1);
    }
}
