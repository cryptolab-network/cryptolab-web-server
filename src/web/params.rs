use std::fmt;
use serde_json::json;
use serde::Deserialize;
use warp::reject;

#[derive(Copy, Clone)]
pub enum ErrorCode {
    InvalidApy = -1000,
    InvalidCommission = -1001,
    InvalidEmailAddress = -1002,
    OperationFailed = -2000,
}

impl ErrorCode {
    pub fn to_int(self) -> i32 {
        self as i32
    }
}

#[derive(Debug)]
pub struct InvalidParam {
    pub message: String,
    pub err_code: i32,
}

impl fmt::Display for InvalidParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = json!({
            "message": self.message,
            "err_code": self.err_code,
        });
        write!(f, "{}", msg.to_string())
    }
}

impl InvalidParam {
    pub fn new(message: &str, err_code: ErrorCode) -> Self{
        InvalidParam {
            message: message.to_string(),
            err_code: err_code.to_int(),
        }
    }
}

#[derive(Debug)]
pub struct OperationFailed {
    pub message: String,
    pub err_code: i32,
}

impl fmt::Display for OperationFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = json!({
            "message": self.message,
            "err_code": self.err_code,
        });
        write!(f, "{}", msg.to_string())
    }
}

impl OperationFailed {
    pub fn new(message: &str, err_code: ErrorCode) -> Self {
        OperationFailed {
            message: message.to_string(),
            err_code: err_code.to_int(),
        }
    }
}

impl reject::Reject for InvalidParam {}
impl reject::Reject for OperationFailed {}

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
    has_verified_identity: Option<bool>,
    has_joined_1kv: Option<bool>,
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
            has_verified_identity: Some(false),
            has_joined_1kv: Some(false),
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

    pub fn has_verified_identity(&self) -> bool {
        self.has_verified_identity.unwrap_or(false)
    }

    pub fn has_joined_1kv(&self) -> bool {
        self.has_joined_1kv.unwrap_or(false)
    }

    pub fn to_db_all_validator_options(&self) -> super::super::db::params::AllValidatorOptions {
        super::super::db::params::AllValidatorOptions {
            size: self.size(),
            page: self.page(),
            apy_min: self.apy_min(),
            apy_max: self.apy_max(),
            commission_min: self.commission_min(),
            commission_max: self.commission_max(),
            has_verified_identity: self.has_verified_identity(),
        }
    }
}

