#[derive(Debug)]
pub enum AppError {
    FetchPageError(reqwest::Error),
    ChatGptError(chatgpt::err::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AppError::FetchPageError(e) => e.to_string(),
            AppError::ChatGptError(e) => e.to_string(),
        };
        f.write_str(&msg)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            AppError::FetchPageError(e) => e,
            AppError::ChatGptError(e) => e,
        })
    }
}
