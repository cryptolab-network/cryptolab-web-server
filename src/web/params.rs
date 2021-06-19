use serde::Deserialize;
use warp::reject;

#[derive(Debug)]
pub struct InvalidParam;

impl reject::Reject for InvalidParam {}

#[derive(Deserialize)]
pub struct ValidDetailOptions {
    pub option: String,
}

#[derive(Deserialize)]
pub struct AllValidatorOptions {
    pub size: Option<u32>,
    pub page: Option<u32>,
    pub apy_min: Option<f32>,
    pub apy_max: Option<f32>,
}