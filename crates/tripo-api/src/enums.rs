//! Shared typed enums. Expanded in Task 5.
use serde::{Deserialize, Serialize};

/// Biological rig class returned by `check_riggable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum RigType {
    /// Two-legged rig (human-like).
    Biped,
    /// Four-legged rig.
    Quadruped,
    /// Six-legged rig.
    Hexapod,
    /// Eight-legged rig.
    Octopod,
    /// Bird-like rig.
    Avian,
    /// Serpent / snake-like rig.
    Serpentine,
    /// Fish / aquatic rig.
    Aquatic,
    /// Other/unclassified.
    Others,
}
