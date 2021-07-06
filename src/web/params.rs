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
    size: Option<u32>,
    page: Option<u32>,
    apy_min: Option<f32>,
    apy_max: Option<f32>,
    commission_min: Option<f32>,
    commission_max: Option<f32>,
}

impl AllValidatorOptions {
    pub fn new() -> Self {
        AllValidatorOptions{
            size: Some(5000),
            page: Some(0),
            apy_min: Some(0.0),
            apy_max: Some(1.0),
            commission_min: Some(0.0),
            commission_max: Some(1.0),
        }
    }

    pub fn size(&self) -> u32 {
        self.size.unwrap_or(5000)
    }

    pub fn page(&self) -> u32 {
        self.page.unwrap_or(0)
    }

    pub fn apy_min(&self) -> f32 {
        self.apy_min.unwrap_or(0.0)
    }

    pub fn apy_max(&self) -> f32 {
        self.apy_max.unwrap_or(1.0)
    }

    pub fn commission_min(&self) -> f32 {
        self.commission_min.unwrap_or(0.0)
    }

    pub fn commission_max(&self) -> f32 {
        self.commission_max.unwrap_or(1.0)
    }

    pub fn to_db_all_validator_options(&self) -> super::super::db::params::AllValidatorOptions {
        super::super::db::params::AllValidatorOptions {
            size: self.size(),
            page: self.page(),
            apy_min: self.apy_min(),
            apy_max: self.apy_max(),
            commission_min: self.commission_min(),
            commission_max: self.commission_max(),
        }
    }
}