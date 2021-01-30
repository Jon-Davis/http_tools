use thiserror::Error;


/// Simple Error type that describes how a Filter failed.
#[derive(PartialEq, Error, Debug, Clone, Copy)]
pub enum FilterError {
    #[error("Filter Failed due to missing or invalid header")]
    FailFilterHeader,
    #[error("Filter Failed due to missing or invalid query parameter")]
    FailFilterQuery,
    #[error("Filter Failed due to invalid path")]
    FailFilterPath,
    #[error("Filter Failed due to incorrect method")]
    FailFilterMethod,
    #[error("Filter Failed due to incorrect scheme")]
    FailFilterScheme,
    #[error("Filter Failed due to incorrect port")]
    FailFilterPort,
    #[error("Filter Failed due to a custom filter failing")]
    FailFilterCustom,
}