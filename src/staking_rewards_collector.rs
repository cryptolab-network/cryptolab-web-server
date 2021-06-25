use std::{fmt, fs::{self, File}, path::PathBuf, process::{Command, Output}, sync::{Arc, Mutex}};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::types::{StashEraReward, StashRewards};

lazy_static! {
  static ref MUTEX: Arc<std::sync::Mutex<i32>> = Arc::new(Mutex::new(0));
}

#[derive(Debug, Clone)]
pub struct SRCError {
    message: String,
    pub err_code: i32,
}

impl fmt::Display for SRCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Staking Rewards Collector error: {}", self.message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StakingRewardsCollector {
  pub start: String,
  pub end: String,
  pub currency: String,
  pub price_data: String,
  pub export_output: String,
  pub addresses: Vec<StakingRewardsAddress>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SRCResult {
  address: String,
	network: String,
	currency: String,
	start_balance: f64,
	first_reward: String,
	last_reward: String,
	annualized_return: Option<f64>,
	current_value_rewards_fiat: f64,
	total_amount_human_readable: f64,
	total_value_fiat: f64,
  data: SRCRewardsData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SRCRewardsData {
  number_rewards_parsed: i32,
  number_of_days: u32,
  list: Vec<SRCDailyRewards>,
}

// "day": "01-01-2020",
// 				"blockNumber": "",
// 				"extrinsicHash": "",
// 				"price": 0,
// 				"volume": 0,
// 				"amountPlanks": 0,
// 				"numberPayouts": 0,
// 				"amountHumanReadable": 0,
// 				"valueFiat": 0

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SRCDailyRewards {
  day: String,
  price: f64,
  volume: f64,
  amount_human_readable: f64,
  value_fiat: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StakingRewardsAddress {
  pub name: String,
  pub address: String,
  pub start_balance: f64,
}

pub struct StakingRewardsReport {
  pub stash: String,
  pub format: String,

  pub exe_dir: String,
}

impl StakingRewardsReport {
  pub fn new(exe_dir: String, stash: String, format: String) -> Self {
    StakingRewardsReport {
      exe_dir: exe_dir,
      stash: stash,
      format: format,
    }
  }

  pub fn get_report(&self) -> Result<String, SRCError> {
    let mut path = PathBuf::new();
    path.push(self.exe_dir.clone());
    path.push(" ".to_string() + &self.stash + "." + &self.format);
    let response_file = File::open(path.clone());
    if let Ok(_) = response_file {
      Ok(fs::read_to_string(path).unwrap())
    } else {
      return Err(SRCError {
        message: "Reward report is not found".to_string(),
        err_code: -20,
      });
    }
  }
}

impl StakingRewardsAddress {
  pub fn new(
    name: String,
    address: String,
    start_balance: f64) -> Self {
      StakingRewardsAddress {
        name: name,
        address: address,
        start_balance: start_balance,
      }
  }
}

impl StakingRewardsCollector {
  pub fn new(
    start: String,
    end: String,
    currency: String,
    price_data: bool,
    addresses: Vec<StakingRewardsAddress>) -> Self {
    StakingRewardsCollector {
      start: start,
      end: end,
      currency: currency,
      price_data: price_data.to_string(),
      addresses: addresses,
      export_output: true.to_string(),
    }
  }

  pub fn call_exe(&self, exe_dir: String) -> Result<StashRewards, SRCError> {
    let mut mutex = MUTEX.lock().unwrap();
    let input = exe_dir.clone() + "/config/userInput.json";
    let file = &File::create(input.clone());
    if let Ok(file) = file {
      let result = serde_json::to_writer_pretty(file, &self);
      if let Ok(()) = result {
        // call exe
        let output  = self._call_exe(&exe_dir);
        // println!("{:?}", output);
        let result = std::str::from_utf8(&output.stdout);
        // parse response
        if let Ok(output) = result {
          if output.starts_with("No rewards found to parse") {
            return Err(SRCError {
              message: "No rewards are found".to_string(),
              err_code: -2,
            });
          }
          let path = exe_dir.clone() + "/ " + &self.addresses[0].address + ".json";
          // println!("{}", path);
          let response_file = File::open(path.clone());
          if let Ok(response_file) = response_file {
            let response: Result<SRCResult, serde_json::Error> = serde_json::from_reader(&response_file);
            if let Ok(mut r) = response {
              r.data.list.reverse(); // make new data on top
              *mutex += 1;
              Ok(self.make_response(&r))
            } else {
              *mutex += 1;
              // println!("---{:?}", response);
              Err(SRCError {
                message: "failed to parse response to SRCResult".to_string(),
                err_code: -9,
              })
            }
          } else {
            *mutex += 1;
              // println!("---{:?}", response);
              Err(SRCError {
                message: "failed to parse response to SRCResult".to_string(),
                err_code: -9,
              })
          }
        } else {
          *mutex += 1;
          Err(SRCError {
            message: "staking rewards collector returned fail".to_string(),
            err_code: -10,
          })
        }
      } else {
        *mutex += 1;
        Err(SRCError {
          message: "failed to write input json to file".to_string(),
          err_code: -11,
        })
      }
    } else {
      *mutex += 1;
      return Err(SRCError {
        message: "failed to create userInput.json".to_string(),
        err_code: -12,
      });
    }
  }

  fn _call_exe<'a>(&self, exe_dir: &'a str) -> Output {
    let output = if cfg!(target_os = "windows") {
      Command::new("cmd")
      .current_dir(exe_dir)
      .args(&["/C", "node src/index.js"])
      .output()
      .expect("failed to execute process")
    } else {
        Command::new("sh")
        .current_dir(exe_dir)
        .arg("-c")
        .arg("node src/index.js")
        .output()
        .expect("failed to execute process")
    };
    output
  }

  fn make_response(&self, src_result: &SRCResult) -> StashRewards {
    let mut era_rewards: Vec<StashEraReward> = vec![];
    let first_date = NaiveDateTime::parse_from_str(&(src_result.first_reward.clone() + " 00:00:00"), "%d-%m-%Y %H:%M:%S").unwrap();
    let last_date = NaiveDateTime::parse_from_str(&(src_result.last_reward.clone() + " 00:00:00"), "%d-%m-%Y %H:%M:%S").unwrap();
    let mut total_in_fiat = 0.0;
    for daily_rewards in src_result.data.list.iter().clone() {
      let date = chrono::NaiveDateTime::parse_from_str(&(daily_rewards.day.clone() + " 00:00:00"), "%d-%m-%Y %H:%M:%S").unwrap();
      if date < first_date || date > last_date {
        continue;
      }
      let date_str = daily_rewards.day.clone() + " 00:00:00";
      era_rewards.push(StashEraReward {
        era: 0,
        amount: daily_rewards.amount_human_readable,
        timestamp: chrono::NaiveDateTime::parse_from_str(date_str.as_str(), "%d-%m-%Y %H:%M:%S")
        .unwrap_or(chrono::NaiveDateTime::from_timestamp(0, 0)).timestamp_millis(),
        price: daily_rewards.price,
        total: daily_rewards.value_fiat,
      });
      total_in_fiat += daily_rewards.value_fiat;
    }

    StashRewards {
      stash: src_result.address.to_string(),
      era_rewards: era_rewards,
      total_in_fiat: total_in_fiat,
    }
  }
}
