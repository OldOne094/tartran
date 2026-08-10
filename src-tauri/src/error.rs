#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    NotFound(String),
    #[error("create failed: {0}")]
    CreateFailed(String),
    #[error("update failed: {0}")]
    UpdateFailed(String),
    #[error("delete failed: {0}")]
    DeleteFailed(String),
    #[error("{0}")]
    Llm(String),
    #[error("export failed: {0}")]
    Export(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::InvalidInput(_) => "INVALID_INPUT",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::CreateFailed(_) => "CREATE_FAILED",
            AppError::UpdateFailed(_) => "UPDATE_FAILED",
            AppError::DeleteFailed(_) => "DELETE_FAILED",
            AppError::Llm(_) => "LLM_ERROR",
            AppError::Export(_) => "EXPORT_FAILED",
            AppError::Io(_) => "IO_ERROR",
            AppError::Db(_) => "DB_ERROR",
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
