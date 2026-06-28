pub fn parse_binds_string(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut binds = Vec::new();
    if s.trim().is_empty() {
        return Ok(binds);
    }
    for part in s.split(',') {
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(format!("Invalid extra bind format: '{}'. Expected 'host:sandbox'.", part));
        }
        binds.push((subparts[0].to_string(), subparts[1].to_string()));
    }
    Ok(binds)
}
