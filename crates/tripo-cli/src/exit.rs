//! Exit codes and error → code mapping.

/// Exit codes used by the CLI.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // `Interrupted` wired up in Task 25.
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

/// Map an `anyhow::Error` to an `ExitCode`.
#[allow(clippy::match_same_arms)] // Explicit Api/Http arm documents intent.
pub fn code_for_error(err: &anyhow::Error) -> ExitCode {
    if err.downcast_ref::<crate::signals::Interrupted>().is_some() {
        return ExitCode::Interrupted;
    }
    if let Some(api_err) = err.downcast_ref::<tripo_api::Error>() {
        return match api_err {
            tripo_api::Error::WaitTimeout(_) => ExitCode::Timeout,
            tripo_api::Error::Api { .. } | tripo_api::Error::Http { .. } => ExitCode::ApiError,
            tripo_api::Error::Io(_) | tripo_api::Error::FileExists(_) => ExitCode::Io,
            tripo_api::Error::TaskFailed(_, _) => ExitCode::TaskNonSuccess,
            tripo_api::Error::MissingApiKey | tripo_api::Error::InvalidApiKey => ExitCode::Usage,
            _ => ExitCode::ApiError,
        };
    }
    eprintln!("error: {err:?}");
    ExitCode::ApiError
}
