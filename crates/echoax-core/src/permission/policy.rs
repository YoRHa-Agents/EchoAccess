#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Sensitivity {
    /// SSH keys, passwords.
    Critical,
    /// Personal configs.
    Private,
    /// General configs.
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionPolicy {
    /// 0600 / DACL: current user Full Control.
    OwnerOnly,
    /// 0644.
    OwnerRwOthersR,
    /// 0755.
    OwnerFull,
    /// 0700 (directories).
    OwnerDir,
}

impl Sensitivity {
    pub fn default_policy(&self) -> PermissionPolicy {
        match self {
            Sensitivity::Critical => PermissionPolicy::OwnerOnly,
            Sensitivity::Private => PermissionPolicy::OwnerOnly,
            Sensitivity::Standard => PermissionPolicy::OwnerRwOthersR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_maps_to_owner_only() {
        assert_eq!(
            Sensitivity::Critical.default_policy(),
            PermissionPolicy::OwnerOnly
        );
    }

    #[test]
    fn private_maps_to_owner_only() {
        assert_eq!(
            Sensitivity::Private.default_policy(),
            PermissionPolicy::OwnerOnly
        );
    }

    #[test]
    fn standard_maps_to_owner_rw_others_r() {
        assert_eq!(
            Sensitivity::Standard.default_policy(),
            PermissionPolicy::OwnerRwOthersR
        );
    }

    #[test]
    fn sensitivity_serde_roundtrip() {
        for s in [
            Sensitivity::Critical,
            Sensitivity::Private,
            Sensitivity::Standard,
        ] {
            let json = serde_json::to_string(&s).expect("serialize");
            let back: Sensitivity = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, s);
        }
    }
}
