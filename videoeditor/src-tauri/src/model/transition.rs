use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionType {
    Cut,
    Fade,
    Crossfade,
    DipBlack,
    DipWhite,
    Slide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionSpec {
    #[serde(rename = "type")]
    pub kind: TransitionType,
    pub duration_ms: u64,
}

impl Default for TransitionSpec {
    fn default() -> Self {
        Self {
            kind: TransitionType::Cut,
            duration_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_type_serializes_kebab_case() {
        let json = serde_json::to_string(&TransitionType::DipBlack).unwrap();
        assert_eq!(json, "\"dip-black\"");
    }

    #[test]
    fn transition_type_deserializes_all_variants() {
        let cases = [
            ("\"cut\"", TransitionType::Cut),
            ("\"fade\"", TransitionType::Fade),
            ("\"crossfade\"", TransitionType::Crossfade),
            ("\"dip-black\"", TransitionType::DipBlack),
            ("\"dip-white\"", TransitionType::DipWhite),
            ("\"slide\"", TransitionType::Slide),
        ];
        for (json, expected) in cases {
            let parsed: TransitionType = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn transition_spec_default_is_cut_zero() {
        let spec = TransitionSpec::default();
        assert_eq!(spec.kind, TransitionType::Cut);
        assert_eq!(spec.duration_ms, 0);
    }
}
