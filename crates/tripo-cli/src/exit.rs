//! Exit codes and error → code mapping.

/// Exit codes used by the CLI.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    /// Command succeeded.
    Success = 0,
    /// Usage error (missing/invalid flags).
    Usage = 2,
    /// API-layer error (non-2xx HTTP, structured envelope error).
    ApiError = 3,
    /// Wait exceeded its timeout.
    Timeout = 4,
    /// Local I/O error (download, filesystem).
    Io = 5,
    /// Task ended with a non-success terminal status.
    TaskNonSuccess = 6,
    /// Interrupted by SIGINT.
    Interrupted = 130,
}

/// Print `err` to stderr and map it to an `ExitCode`.
#[allow(clippy::match_same_arms)] // Explicit Api/Http arm documents intent.
pub fn code_for_error(err: &anyhow::Error) -> ExitCode {
    if err.downcast_ref::<crate::signals::Interrupted>().is_some() {
        return ExitCode::Interrupted;
    }
    eprintln!("error: {err:?}");
    let Some(api_err) = err.downcast_ref::<tripo_api::Error>() else {
        return ExitCode::ApiError;
    };
    if let Some(hint) = hint_for(api_err) {
        eprintln!("hint: {hint}");
    }
    match api_err {
        tripo_api::Error::WaitTimeout(_) => ExitCode::Timeout,
        tripo_api::Error::Api { .. } | tripo_api::Error::Http { .. } => ExitCode::ApiError,
        tripo_api::Error::Io(_)
        | tripo_api::Error::File { .. }
        | tripo_api::Error::FileExists(_) => ExitCode::Io,
        tripo_api::Error::TaskFailed { .. } => ExitCode::TaskNonSuccess,
        tripo_api::Error::MissingApiKey
        | tripo_api::Error::InvalidApiKey
        | tripo_api::Error::InvalidRequest(_) => ExitCode::Usage,
        _ => ExitCode::ApiError,
    }
}

/// CLI-specific remedy for errors whose library message is frontend-neutral.
fn hint_for(err: &tripo_api::Error) -> Option<&'static str> {
    match err {
        tripo_api::Error::FileExists(_) => Some("pass --force to overwrite"),
        tripo_api::Error::MissingApiKey => Some("set TRIPO_API_KEY or pass --api-key"),
        _ => None,
    }
}
