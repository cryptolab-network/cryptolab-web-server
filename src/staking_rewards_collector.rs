use std::{fmt, fs::{File}, process::{Command, Output}, sync::{Arc, Mutex}};

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::types::{StashEraReward, StashRewards};

lazy_static! {
  static ref MUTEX: Arc<std::sync::Mutex<i32>> = Arc::new(Mutex::new(0));
}

#[derive(Debug, Clone)]
pub struct SRCError {
    message: String,
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
    start: DateTime::<Utc>,
    end: DateTime::<Utc>,
    currency: String,
    price_data: bool,
    addresses: Vec<StakingRewardsAddress>) -> Self {
    StakingRewardsCollector {
      start: start.format("%Y-%m-%d").to_string(),
      end: end.format("%Y-%m-%d").to_string(),
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
        if let Ok(_) = result {
          let path = exe_dir.clone() + "\\ " + &self.addresses[0].address + ".json";
          // println!("{}", path);
          let response = serde_json::from_reader(&File::open(path.clone()).unwrap());
          if let Ok(response) = response {
            *mutex += 1;
            Ok(self.make_response(&response))
          } else {
            *mutex += 1;
            // println!("---{:?}", response);
            Err(SRCError {
              message: "failed to parse response to SRCResult".to_string(),
            })
          }
        } else {
          *mutex += 1;
          Err(SRCError {
            message: "staking rewards collector returned fail".to_string(),
          })
        }
      } else {
        *mutex += 1;
        Err(SRCError {
          message: "failed to write input json to file".to_string(),
        })
      }
    } else {
      *mutex += 1;
      return Err(SRCError {
        message: "failed to create userInput.json".to_string(),
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
    let mut drop_zero = true;
    for daily_rewards in src_result.data.list.iter().clone() {
      if drop_zero {
        if daily_rewards.amount_human_readable == 0.0 {
          continue;
        }
      }
      drop_zero = false;
      let date_str = daily_rewards.day.clone() + " 00:00:00";
      era_rewards.push(StashEraReward {
        era: 0,
        amount: daily_rewards.amount_human_readable,
        timestamp: chrono::NaiveDateTime::parse_from_str(date_str.as_str(), "%d-%m-%Y %H:%M:%S")
        .unwrap_or(chrono::NaiveDateTime::from_timestamp(0, 0)).timestamp_millis(),
        price: daily_rewards.price,
        total: daily_rewards.value_fiat,
      });
    }

    StashRewards {
      stash: src_result.address.to_string(),
      era_rewards: era_rewards,
    }
  }
}
