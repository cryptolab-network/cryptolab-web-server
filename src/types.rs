use std::fmt::Display;
use std::str::FromStr;
use serde_json::Value;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotApiValidators {
    pub valid_detail_all: ValidatorDetailAll,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDetailAll {
    pub valid: Vec<ValidatorInfo>
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
    identity: Identity
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
            let result = u128::from_str_radix(&s[2..], 16);
            result.map_err(de::Error::custom)?
        },
        Value::Number(num) => {
            u128::from_str(num.to_string().as_str()).map_err(de::Error::custom)?
        },
        _ => return Err(de::Error::custom("wrong type"))
    })
}