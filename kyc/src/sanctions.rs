pub struct SanctionsChecker {
    blocklist: Vec<String>,
}

impl SanctionsChecker {
    pub fn new(blocklist: Vec<String>) -> Self {
        Self { blocklist }
    }

    pub fn is_jurisdiction_blocked(&self, jurisdiction: &str) -> bool {
        let j = jurisdiction.to_lowercase().trim().to_string();
        self.blocklist.iter().any(|b| b == &j)
    }

    pub fn check(&self, jurisdiction: &str) -> anyhow::Result<()> {
        if self.is_jurisdiction_blocked(jurisdiction) {
            anyhow::bail!("Jurisdiction {} is under sanctions", jurisdiction);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanctions_blocklist() {
        let checker = SanctionsChecker::new(vec!["ir".into(), "kp".into(), "cu".into()]);
        assert!(checker.is_jurisdiction_blocked("ir"));
        assert!(checker.is_jurisdiction_blocked("KP"));
        assert!(!checker.is_jurisdiction_blocked("us"));
        assert_eq!(checker.check("cu").is_err(), true);
        assert_eq!(checker.check("us").is_ok(), true);
    }
}
