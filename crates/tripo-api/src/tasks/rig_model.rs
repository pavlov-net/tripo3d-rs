//! `rig_model` task variant. Endpoint: `POST /animations/rig`.

use serde::{Deserialize, Serialize};

use crate::enums::{RigOutputFormat, RigSpec, RigType};
use crate::error::{Error, Result};
use crate::versions;

/// Request body for `POST /animations/rig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RigModelRequest {
    /// Model source: `task_id`, `file_token`, or URL (GLB, FBX, OBJ, or STL).
    pub input: String,
    /// Rigging model version; see `versions::rig`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Output file format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_format: Option<RigOutputFormat>,
    /// Rig classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rig_type: Option<RigType>,
    /// Skeleton spec (biped only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<RigSpec>,
}

/// Known rigger versions, oldest first.
#[derive(PartialEq, PartialOrd)]
enum Tier {
    V1_0,
    V2_5,
    V3_0,
}

impl Tier {
    /// `None` (server default) is v1.0. Unrecognized strings map to `None`
    /// and skip validation so newer server versions aren't blocked.
    fn of(model: Option<&str>) -> Option<Self> {
        match model {
            None | Some(versions::rig::V1_0) => Some(Self::V1_0),
            Some(versions::rig::V2_5) => Some(Self::V2_5),
            Some(versions::rig::V3_0) => Some(Self::V3_0),
            Some(_) => None,
        }
    }
}

impl RigModelRequest {
    /// Reject `rig_type` / `spec` values the chosen rigger version doesn't
    /// support. The server answers those with `code: 1004` ("one or more of
    /// your parameter is invalid") and no useful detail.
    pub(crate) fn validate(&self) -> Result<()> {
        use versions::rig::{V2_5, V3_0};

        let Some(tier) = Tier::of(self.model.as_deref()) else {
            return Ok(());
        };
        if let Some(spec @ (RigSpec::ActorCore | RigSpec::Unity | RigSpec::Unreal | RigSpec::Vrm)) =
            self.spec
            && tier < Tier::V3_0
        {
            return Err(Error::InvalidRequest(format!(
                "spec {} requires model {V3_0}",
                spec.as_str(),
            )));
        }
        match self.rig_type {
            Some(RigType::Others) if tier < Tier::V3_0 => Err(Error::InvalidRequest(format!(
                "rig_type others requires model {V3_0}"
            ))),
            Some(t) if t != RigType::Biped && tier == Tier::V1_0 => {
                Err(Error::InvalidRequest(format!(
                    "rig_type {} requires model {V2_5} or {V3_0} — the default/v1.0 rigger only supports biped",
                    t.as_str(),
                )))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use versions::rig::{V1_0, V2_5, V3_0};

    fn req(rig_type: Option<RigType>, model: Option<&str>) -> RigModelRequest {
        RigModelRequest {
            input: "task_id".into(),
            model: model.map(str::to_owned),
            out_format: None,
            rig_type,
            spec: None,
        }
    }

    fn with_spec(spec: RigSpec, model: Option<&str>) -> RigModelRequest {
        RigModelRequest {
            spec: Some(spec),
            ..req(Some(RigType::Biped), model)
        }
    }

    #[test]
    fn biped_with_default_version_ok() {
        req(Some(RigType::Biped), None).validate().unwrap();
    }

    #[test]
    fn no_rig_type_any_version_ok() {
        for model in [None, Some(V1_0), Some(V2_5), Some(V3_0)] {
            req(None, model).validate().unwrap();
        }
    }

    #[test]
    fn avian_with_v1_is_rejected() {
        let err = req(Some(RigType::Avian), None).validate().unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest(ref msg) if msg.contains("v2.5") && msg.contains("biped")),
            "{err}"
        );
        let err = req(Some(RigType::Avian), Some(V1_0))
            .validate()
            .unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(_)));
    }

    #[test]
    fn avian_with_v2_5_or_v3_is_accepted() {
        req(Some(RigType::Avian), Some(V2_5)).validate().unwrap();
        req(Some(RigType::Avian), Some(V3_0)).validate().unwrap();
    }

    #[test]
    fn non_biped_rig_types_all_rejected_on_v1() {
        for t in [
            RigType::Avian,
            RigType::Quadruped,
            RigType::Hexapod,
            RigType::Octopod,
            RigType::Serpentine,
            RigType::Aquatic,
            RigType::Others,
        ] {
            req(Some(t), None).validate().unwrap_err();
        }
    }

    #[test]
    fn others_requires_v3() {
        let err = req(Some(RigType::Others), Some(V2_5))
            .validate()
            .unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest(ref msg) if msg.contains(V3_0)),
            "{err}"
        );
        req(Some(RigType::Others), Some(V3_0)).validate().unwrap();
    }

    #[test]
    fn v3_specs_require_v3() {
        for spec in [
            RigSpec::ActorCore,
            RigSpec::Unity,
            RigSpec::Unreal,
            RigSpec::Vrm,
        ] {
            for model in [None, Some(V1_0), Some(V2_5)] {
                let err = with_spec(spec, model).validate().unwrap_err();
                assert!(
                    matches!(err, Error::InvalidRequest(ref msg) if msg.contains(spec.as_str()) && msg.contains(V3_0)),
                    "{err}"
                );
            }
            with_spec(spec, Some(V3_0)).validate().unwrap();
        }
    }

    #[test]
    fn legacy_specs_ok_on_every_version() {
        for spec in [RigSpec::Tripo, RigSpec::Mixamo] {
            for model in [None, Some(V1_0), Some(V2_5), Some(V3_0)] {
                with_spec(spec, model).validate().unwrap();
            }
        }
    }

    #[test]
    fn unknown_model_passes_through() {
        req(Some(RigType::Others), Some("v3.1-20270101"))
            .validate()
            .unwrap();
        with_spec(RigSpec::Vrm, Some("v3.1-20270101"))
            .validate()
            .unwrap();
    }
}
