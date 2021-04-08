use std::fmt::Display;
use std::str::FromStr;
use serde_json::Value;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotApiValidators {
    pub valid_detail_all: Option<ValidatorDetailAll>,
    #[serde(rename = "validDetail")]
    pub valid_detail_1kv: Option<ValidatorDetail1kv>,
    #[serde(rename = "valid")]
    pub valid: Option<Validator1kvSimple>,
    pub nominators: Option<Vec<NominatorNomination>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDetail1kv {
    pub active_era: Option<u32>,
    pub validator_count: Option<u32>,
    pub elected_count: Option<u32>,
    pub election_rate: Option<f32>,
    pub valid: Vec<ValidatorInfo1kv>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Validator1kvSimple {
    pub active_era: Option<u32>,
    pub validator_count: Option<u32>,
    pub elected_count: Option<u32>,
    pub election_rate: Option<f32>,
    pub valid: Vec<ValidatorInfo1kvSimple>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDetailAll {
    pub valid: Vec<ValidatorInfo>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo1kv {
    aggregate: Aggregate,
    rank: u32,
    inclusion: f32,
    name: String,
    stash: String,
    elected: bool,
    active_nominators: u32,
    total_nominators: u32,
    staking_info: Option<StakingInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo1kvSimple {
    aggregate: Aggregate,
    rank: u32,
    inclusion: f32,
    name: String,
    stash: String,
    elected: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StakingInfo {
    account_id: String,
    exposure: Exposure,
    nominators: Vec<Nominator>,
    staking_ledger: StakingLedger,
    stash_id: String,
    validator_prefs: ValidatorPrefs,
    identity: Identity,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Aggregate {
    total: f32,
    aggregate: f32,
    inclusion: f32,
    discovered: f32,
    nominated: f32,
    rank: f32,
    unclaimed: f32,
    randomness: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo {
    account_id: String,
    controller_id: String,
    exposure: Exposure,
    nominators: Vec<Nominator>,
    staking_ledger: StakingLedger,
    validator_prefs: ValidatorPrefs,
    identity: Identity,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exposure {
    #[serde(deserialize_with = "from_hex")]
    total: u128,
    #[serde(deserialize_with = "from_hex")]
    own: u128,
    others: Vec<Others>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Others {
    who: String,
    #[serde(deserialize_with = "from_hex")]
    value: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nominator {
    address: String,
    balance: Balance
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NominatorNomination {
    account_id: String,
    balance: Balance,
    targets: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    #[serde(deserialize_with = "from_hex")]
    locked_balance: u128,
    #[serde(deserialize_with = "from_hex")]
    free_balance: u128
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StakingLedger {
    stash: String,
    #[serde(deserialize_with = "from_hex")]
    total: u128,
    #[serde(deserialize_with = "from_hex")]
    active: u128
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidatorPrefs {
    commission: u64,
    blocked: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Identity {
    display: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorNominationInfo {
    id: String,
    status_change: StatusChange,
    identity: Identity,
    info: NominationInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorNominationTrend {
    id: String,
    status_change: StatusChange,
    identity: Identity,
    info: Vec<NominationInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusChange {
    commission: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NominationInfo {
    nominators: Vec<Nominator>,
    era: u32,
    exposure: Exposure,
    commission: f32,
    apy: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub active_era: u32,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    println!("{}", s);
    T::from_str(&s).map_err(de::Error::custom)
}

fn from_hex<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => {
            let mut result = Ok(0);
            if s.len() > 3 {
                result = u128::from_str_radix(&s[2..], 16);
            }
            result.map_err(de::Error::custom)?
        },
        Value::Number(num) => {
            u128::from_str(num.to_string().as_str()).map_err(de::Error::custom)?
        },
        _ => return Err(de::Error::custom("wrong type"))
    })
}