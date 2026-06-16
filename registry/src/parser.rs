use crate::provider::VerificationRecord;

pub struct RegistryParser;

impl RegistryParser {
    pub fn parse(json: &str) -> anyhow::Result<VerificationRecord> {
        let v: serde_json::Value = serde_json::from_str(json)?;

        let registry = v["registry"].as_str().unwrap_or("unknown").to_string();
        let cert_id = v["cert_id"].as_str().unwrap_or("unknown").to_string();
        let project_id = v["project_id"].as_str().unwrap_or(&cert_id).to_string();
        let project_name = v["project_name"].as_str().unwrap_or("Unknown").to_string();
        let project_type = v["project_type"].as_str().map(|s| s.to_string());
        let location = v["location"].as_str().map(|s| s.to_string());
        let vintage_year = v["vintage_year"].as_i64().map(|n| n as i32);

        let timestamp = v["timestamp"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let suspended = v["suspended"].as_bool().unwrap_or(false);

        Ok(VerificationRecord {
            registry,
            cert_id,
            project_id,
            project_name,
            project_type,
            location,
            vintage_year,
            timestamp,
            suspended,
        })
    }

    pub fn print(record: &VerificationRecord) -> String {
        serde_json::to_string(record).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_verra() -> &'static str {
        r#"{
            "registry": "verra",
            "cert_id": "VCS-1234",
            "project_id": "VCS-1234",
            "project_name": "Amazon Reforestation",
            "project_type": "forestry",
            "location": "Brazil",
            "vintage_year": 2023,
            "timestamp": "2024-01-15T00:00:00Z",
            "suspended": false
        }"#
    }

    fn sample_gold_standard() -> &'static str {
        r#"{
            "registry": "gold_standard",
            "cert_id": "GS-5678",
            "project_id": "GS-5678",
            "project_name": "Wind Farm Kenya",
            "project_type": "renewable_energy",
            "location": "Kenya",
            "vintage_year": 2024,
            "timestamp": "2024-03-20T00:00:00Z",
            "suspended": false
        }"#
    }

    fn sample_acr() -> &'static str {
        r#"{
            "registry": "acr",
            "cert_id": "ACR-9012",
            "project_id": "ACR-9012",
            "project_name": "Methane Capture",
            "project_type": "industrial",
            "location": "USA",
            "vintage_year": 2023,
            "timestamp": "2024-06-10T00:00:00Z",
            "suspended": false
        }"#
    }

    #[test]
    fn test_roundtrip_verra() {
        let parsed = RegistryParser::parse(sample_verra()).unwrap();
        let printed = RegistryParser::print(&parsed);
        let reparsed = RegistryParser::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_roundtrip_gold_standard() {
        let parsed = RegistryParser::parse(sample_gold_standard()).unwrap();
        let printed = RegistryParser::print(&parsed);
        let reparsed = RegistryParser::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_roundtrip_acr() {
        let parsed = RegistryParser::parse(sample_acr()).unwrap();
        let printed = RegistryParser::print(&parsed);
        let reparsed = RegistryParser::parse(&printed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_invalid_json() {
        assert!(RegistryParser::parse("not json").is_err());
    }
}
