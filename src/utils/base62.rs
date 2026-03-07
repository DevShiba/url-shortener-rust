use crate::errors::AppError;
use harsh::Harsh;

#[derive(Debug)]
pub struct Shortener {
    harsh: Harsh,
}

impl Shortener {
    pub fn new(salt: &str, min_length: usize) -> Result<Self, AppError> {
        let harsh = Harsh::builder()
            .salt(salt)
            .length(min_length)
            .build()
            .map_err(|e| AppError::Config(format!("failed to build Hashids encoder: {e}")))?;

        Ok(Self { harsh })
    }

    pub fn encode(&self, id: u64) -> Result<String, AppError> {
        let code = self.harsh.encode(&[id]);

        if code.is_empty() {
            return Err(AppError::Encoding);
        }

        Ok(code)
    }
}
