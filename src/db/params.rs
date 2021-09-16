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
    pub has_verified_identity: bool,
}

#[derive(Deserialize)]
pub struct EventFilterOptions {
    pub from_era: u32,
    pub to_era: u32,
}
