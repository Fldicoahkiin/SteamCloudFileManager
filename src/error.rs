use std::sync::PoisonError;

#[derive(Debug)]
pub enum AppError {
    SteamNotConnected,
    SteamLockError(String),
    InvalidAppId,
    IoError(std::io::Error),
    AnyhowError(anyhow::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::SteamNotConnected => write!(f, "Steam 未连接"),
            AppError::SteamLockError(msg) => write!(f, "Steam 管理器锁错误: {}", msg),
            AppError::InvalidAppId => write!(f, "无效的 App ID"),
            AppError::IoError(e) => write!(f, "IO 错误: {}", e),
            AppError::AnyhowError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::AnyhowError(err)
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError::SteamLockError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
