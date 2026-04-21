//! `rig_model` task variant. Wire `type`: `animate_rig`.

use serde::{Deserialize, Serialize};

use crate::enums::{RigOutputFormat, RigSpec, RigType};
use crate::error::{Error, Result};
use crate::versions;

/// Request body for `rig_model`. Wire `type`: `animate_rig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RigModelRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Output file format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_format: Option<RigOutputFormat>,
    /// Rig classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rig_type: Option<RigType>,
    /// Skeleton spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<RigSpec>,
}

impl RigModelRequest {
    /// Reject non-biped `rig_type` with a rigger version that only supports
    /// biped. The server's default rigger (`v1.0-20240301`) is biped-only, so
    /// any non-biped `rig_type` requires an explicit `--model-version v2.0-20250506`
    /// or `v2.5-20260210`. Without this check, the server returns `error_code:
    /// 1004` ("one or more of your parameter is invalid") with no `error_msg`.
    pub(crate) fn validate(&self) -> Result<()> {
        let non_biped = matches!(
            self.rig_type,
            Some(
                RigType::Avian
                    | RigType::Quadruped
                    | RigType::Hexapod
                    | RigType::Octopod
                    | RigType::Serpentine
                    | RigType::Aquatic
                    | RigType::Others
            )
        );
        let biped_only_version = match self.model_version.as_deref() {
            None | Some(versions::rig::V1_0) => true,
            Some(_) => false,
        };
        if non_biped && biped_only_version {
            return Err(Error::InvalidRequest(format!(
                "rig_type {:?} requires model_version {} or {} — the default/v1.0 rigger only supports biped",
                self.rig_type.as_ref().unwrap(),
                versions::rig::V2_5,
                versions::rig::V2_0,
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(rig_type: Option<RigType>, model_version: Option<&str>) -> RigModelRequest {
        RigModelRequest {
            original_model_task_id: "task_id".into(),
            model_version: model_version.map(str::to_owned),
            out_format: None,
            rig_type,
            spec: None,
        }
    }

    #[test]
    fn biped_with_default_version_ok() {
        req(Some(RigType::Biped), None).validate().unwrap();
    }

    #[test]
    fn no_rig_type_any_version_ok() {
        req(None, None).validate().unwrap();
        req(None, Some(versions::rig::V1_0)).validate().unwrap();
        req(None, Some(versions::rig::V2_5)).validate().unwrap();
    }

    #[test]
    fn avian_with_v1_is_rejected() {
        let err = req(Some(RigType::Avian), None).validate().unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest(ref msg) if msg.contains("v2.5") && msg.contains("biped")),
            "{err}"
        );
        let err = req(Some(RigType::Avian), Some(versions::rig::V1_0))
            .validate()
            .unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(_)));
    }

    #[test]
    fn avian_with_v2_is_accepted() {
        req(Some(RigType::Avian), Some(versions::rig::V2_0))
            .validate()
            .unwrap();
        req(Some(RigType::Avian), Some(versions::rig::V2_5))
            .validate()
            .unwrap();
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
            req(Some(t.clone()), None).validate().unwrap_err();
        }
    }
}
