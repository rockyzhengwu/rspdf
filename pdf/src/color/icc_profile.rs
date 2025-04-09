use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ICCProfile {}

impl ICCProfile {
    pub fn try_new(data: &[u8]) -> Result<Self> {
        Ok(ICCProfile {})
    }
}
