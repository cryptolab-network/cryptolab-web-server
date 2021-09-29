use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use validator::Validate;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotApiValidators {
    pub valid_detail_all: Option<ValidatorDetailAll>,
    #[serde(rename = "validDetail")]
    pub valid_detail_1kv: Option<ValidatorDetail1kv>,
    #[serde(rename = "valid")]
    pub valid: Option<Validator1kvSimple>,
    pub nominators: Option<Vec<NominatorNomination>>,
    #[serde(rename = "onekvNominators")]
    pub one_kv_nominators: Option<OneKvNominators>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDetail1kv {
    pub active_era: Option<u32>,
    pub validator_count: Option<u32>,
    pub elected_count: Option<u32>,
    pub election_rate: Option<f32>,
    pub valid: Vec<ValidatorInfo1kv>,
    pub modified_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Validator1kvSimple {
    pub active_era: Option<u32>,
    pub validator_count: Option<u32>,
    pub elected_count: Option<u32>,
    pub election_rate: Option<f32>,
    pub valid: Vec<ValidatorInfo1kvSimple>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDetailAll {
    pub valid: Vec<ValidatorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Validity {
    #[serde(rename="type")]
    pub validity_type: String,
    pub valid: bool,
    pub details: String,
    pub updated: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo1kv {
    #[serde(default)]
    aggregate: Option<Aggregate>,
    rank: i32,
    inclusion: f32,
    name: String,
    pub stash: String,
    elected: bool,
    active_nominators: u32,
    total_nominators: u32,
    staking_info: Option<StakingInfo>,
    nominated_at: String,
    #[serde(deserialize_with = "from_optional_hex")]
    self_stake: Option<u128>,
    pub valid: Option<bool>,
    pub validity: Vec<Validity>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo1kvSimple {
    #[serde(default)]
    aggregate: Option<Aggregate>,
    rank: u32,
    inclusion: f32,
    name: String,
    stash: String,
    elected: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StakingInfo {
    // account_id: String,
    // exposure: Exposure,
    // nominators: Vec<Nominator>,
    staking_ledger: StakingLedger,
    #[serde(alias = "stash", alias = "stashId")]
    stash_id: String,
    validator_prefs: ValidatorPrefs,
    // identity: Identity,
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

impl Default for Aggregate {
    fn default() -> Self {
        Aggregate {
            total: 0.0,
            aggregate: 0.0,
            inclusion: 0.0,
            discovered: 0.0,
            nominated: 0.0,
            rank: 0.0,
            unclaimed: 0.0,
            randomness: 0.0,
        }
    }
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
    unclaimed_eras: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Exposure {
    #[serde(deserialize_with = "from_hex")]
    total: u128,
    #[serde(deserialize_with = "from_hex")]
    own: u128,
    others: Vec<Others>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Others {
    who: String,
    #[serde(deserialize_with = "from_hex")]
    value: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nominator {
    pub address: String,
    pub balance: Option<Balance>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NominatorNomination {
    #[serde(alias = "address", alias = "accountId")]
    pub account_id: String,
    pub balance: Balance,
    pub targets: Vec<String>,
    pub rewards: Option<StashRewards>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    #[serde(deserialize_with = "from_hex")]
    pub(crate) locked_balance: u128,
    #[serde(deserialize_with = "from_hex")]
    pub(crate) free_balance: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StakingLedger {
    #[serde(alias = "stash", alias = "stashId")]
    stash: String,
    #[serde(deserialize_with = "from_hex")]
    total: u128,
    #[serde(deserialize_with = "from_hex")]
    active: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidatorPrefs {
    commission: u64,
    blocked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    display: Option<String>,
    parent: Option<String>,
    sub: Option<String>,
    is_verified: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorNominationInfo {
    pub id: String,
    status_change: StatusChange,
    identity: Option<Identity>,
    info: NominationInfoSimple,
    staker_points: Option<Vec<StakerPoint>>,
    average_apy: Option<f32>,
    slashes: Vec<ValidatorSlash>,
    #[serde(alias = "block_nomination", alias = "blocked")]
    block_nomination: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorNominationTrend {
    id: String,
    status_change: StatusChange,
    identity: Option<Identity>,
    average_apy: Option<f32>,
    staker_points: Option<Vec<StakerPoint>>,
    pub info: Vec<NominationInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StakerPoint {
    era: u32,
    points: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorTotalReward {
    start: i32,
    end: i32,
    total: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StashRewards {
    #[serde(alias = "id")]
    pub stash: String,
    pub era_rewards: Vec<StashEraReward>,
    pub total_in_fiat: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StashEraReward {
    pub era: i32,
    pub amount: f64,
    #[serde(default, deserialize_with = "from_float")]
    pub timestamp: i64,
    pub price: Option<f64>,
    pub total: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CBStashEraReward {
    pub era: i32,
    pub amount: f64,
    #[serde(default, deserialize_with = "from_float")]
    pub timestamp: i64,
    #[serde(alias = "stash")]
    pub address: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoinPrice {
    pub price: f64,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusChange {
    commission: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NominationInfo {
    pub nominators: Option<Vec<Nominator>>,
    pub nominator_count: u32,
    pub era: u32,
    exposure: Exposure,
    commission: f32,
    apy: f32,
    unclaimed_eras: Option<Vec<i32>>,
    #[serde(default, deserialize_with = "from_hex")]
    total: u128,
    #[serde(default, deserialize_with = "from_optional_hex")]
    self_stake: Option<u128>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NominationInfoSimple {
    pub nominators: Option<Vec<Nominator>>,
    pub nominator_count: u32,
    pub era: u32,
    exposure: Exposure,
    commission: f32,
    apy: f32,
    unclaimed_eras: Option<Vec<i32>>,
    #[serde(default, deserialize_with = "from_hex")]
    total: u128,
    #[serde(deserialize_with = "from_optional_hex")]
    self_stake: Option<u128>,
}

impl NominationInfo {
    pub fn set_nominators(&mut self, nominators: Vec<Nominator>) {
        self.nominators = Some(nominators);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub active_era: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OneKvNominators {
    pub active_era: u32,
    pub nominators: Vec<OneKvNominator>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OneKvNominator {
    current: Vec<OneKvNominated>,
    last_nomination: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OneKvNominated {
    stash: String,
    #[serde(deserialize_with = "parse_stash_name")]
    name: String,
    #[serde(deserialize_with = "parse_elected")]
    elected: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorSlash {
    address: String,
    #[serde(deserialize_with = "from_hex")]
    total: u128,
    others: Vec<ValidatorSlashNominator>,
    era: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorStalePayoutEvent {
    address: String,
    unclaimed_payout_eras: Vec<u32>,
    era: u32,
}

#[derive(Serialize, Deserialize,  Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorSlashNominator {
    address: String,
    #[serde(deserialize_with = "from_hex")]
    value: u128,
}

// fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
//     where T: FromStr,
//           T::Err: Display,
//           D: Deserializer<'de>
// {
//     let s = String::deserialize(deserializer)?;
//     T::from_str(&s).map_err(de::Error::custom)
// }

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
            // println!("{:?}", result);
            result.map_err(de::Error::custom)?
        }
        Value::Number(num) => {
            let result = u128::from_str(num.to_string().as_str());
            // println!("{:?}", result);
            result.map_err(de::Error::custom)?
        }
        _ => return Err(de::Error::custom("wrong type")),
    })
}

fn from_optional_hex<'de, D>(deserializer: D) -> Result<Option<u128>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer);
    if let Ok(v) = v {
        match v {
            Value::String(s) => {
                let mut result = Ok(0);
                if s.len() > 3 {
                    result = u128::from_str_radix(&s[2..], 16);
                }
                Ok(Some(result.map_err(de::Error::custom)?))
            }
            Value::Number(num) => {
                Ok(Some(u128::from_str(num.to_string().as_str()).map_err(de::Error::custom)?))
            }
            a => {
                println!("{:?}", a);
                Err(de::Error::custom("wrong type"))
            },
        }
    } else {
        println!("{:?}", v);
        Ok(Some(0))
    }
    
}

fn parse_stash_name<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_else(|| "N/A".to_string()))
}

fn parse_elected<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or(false))
}

fn from_float<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer);
    if let Ok(v) = v {
        match v {
            Value::Number(num) => {
                Ok(i64::from_str(num.to_string().as_str()).map_err(de::Error::custom)?)
            }
            a => {
                println!("{:?}", a);
                Err(de::Error::custom("wrong type"))
            },
        }
    } else {
        println!("{:?}", v);
        Ok(0)
    }
    
}


#[derive(Deserialize)]
pub enum NominationStrategy {
    Default = 0,
    LowRisk = 1,
    HighApy = 2,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NominationOptions {
    pub stash: String,
    pub validators: Vec<String>,
    pub amount: u128,
    pub strategy: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NominationResultOptions {
    pub tag: String,
    pub extrinsic_hash: String,
} 

#[derive(Deserialize, Validate, Debug)]
pub struct NewsletterSubscriberOptions {
    #[validate(email)]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorCommission {
    pub address: String,
    pub era: u32,
    pub commission_from: f32,
    pub commission_to: f32,
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StakingEvents {
    pub commissions: Vec<ValidatorCommission>,
    pub slashes: Vec<ValidatorSlash>,
    pub inactive: Vec<u32>,
    pub stale_payouts: Vec<ValidatorStalePayoutEvent>,
    pub payouts: Vec<CBStashEraReward>,
    pub kicks: Vec<KickEvent>,
    pub chills: Vec<ChillEvent>,
    pub over_subscribes: Vec<OverSubscribeEventOutput>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KickEvent {
    pub address: String,
    pub nominator: String,
    pub era: u32,
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChillEvent {
    pub address: String,
    pub era: u32,
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OverSubscribeEvent {
    pub nominators: Vec<IndividualExposure>,
    pub address: String,
    pub era: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OverSubscribeEventOutput {
    pub nominator: String,
    pub amount: String,
    pub address: String,
    pub era: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IndividualExposure {
    pub who: String,
    pub value: String,
}