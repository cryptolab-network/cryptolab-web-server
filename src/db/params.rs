use serde::Deserialize;

#[derive(Deserialize)]
pub struct ValidDetailOptions {
    pub option: String,
}

#[derive(Deserialize)]
pub struct AllValidatorOptions {
    pub size: u32,
    pub page: u32,
    pub apy_min: f32,
    pub apy_max: f32,
    pub commission_min: f32,
    pub commission_max: f32,
}
