/* use  **************************************************************************************************/

use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use csv;
use encoding_rs::SHIFT_JIS;
use encoding_rs_io::DecodeReaderBytesBuilder;
use env_logger::init as init_env_logger;
use log::{info, warn};
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter};

/* mod  **************************************************************************************************/

/* type alias  *******************************************************************************************/

/* global const  *****************************************************************************************/

/* trait  ************************************************************************************************/

/* enum  *************************************************************************************************/

/* struct  ***********************************************************************************************/

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Input CSV file. If not specified, read from stdin.
  #[arg(short = 'i', long = "input")]
  input: Option<String>,

  /// Output CSV file. If not specified, write to stdout.
  #[arg(short = 'o', long = "output")]
  output: Option<String>,
}

/* unsafe impl standard traits  **************************************************************************/

/* impl standard traits  *********************************************************************************/

/* impl custom traits  ***********************************************************************************/

/* impl  *************************************************************************************************/

/* fn  ***************************************************************************************************/

fn conv_time(x: &str) -> Result<NaiveDateTime> {
  // フォーマット: "MM/DD/YY, HH:MM:SS AM/PM GMT+X"
  // 例: "11/25/24, 11:28:34 AM GMT+9"

  let x = x.trim();
  let parts: Vec<&str> = x.split(',').collect();
  if parts.len() != 2 {
    return Err(anyhow!(
      "Invalid format: expected two parts separated by a comma"
    ));
  }

  let date_part = parts[0].trim(); // "11/25/24"
  let time_part = parts[1].trim(); // "11:28:34 AM GMT+9"

  // 日付パース
  let date_split: Vec<&str> = date_part.split('/').collect();
  if date_split.len() != 3 {
    return Err(anyhow!("Invalid date format"));
  }
  let month: u32 = date_split[0].parse()?;
  let day: u32 = date_split[1].parse()?;
  // "24"を20xx年とみなす
  let year: i32 = 2000 + date_split[2].parse::<i32>()?;

  // 時刻部分のパース
  let time_split: Vec<&str> = time_part.split_whitespace().collect();
  if time_split.len() < 3 {
    return Err(anyhow!("Invalid time format"));
  }

  let hms = time_split[0]; // "11:28:34"
  let ampm = time_split[1]; // "AM" or "PM"
                            // "GMT+9"は今回は無視

  let hms_split: Vec<&str> = hms.split(':').collect();
  if hms_split.len() != 3 {
    return Err(anyhow!("Invalid H:M:S format"));
  }

  let mut hour: u32 = hms_split[0].parse()?;
  let minute: u32 = hms_split[1].parse()?;
  let second: u32 = hms_split[2].parse()?;

  // AM/PM処理を24時間表記へ変換
  match ampm {
    "AM" => {
      if hour == 12 {
        hour = 0;
      }
    }
    "PM" => {
      if hour != 12 {
        hour += 12;
      }
    }
    _ => return Err(anyhow!("Invalid AM/PM indicator")),
  }

  let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| anyhow!("Invalid date"))?;
  let ndt = date
    .and_hms_opt(hour, minute, second)
    .ok_or_else(|| anyhow!("Invalid time"))?;

  Ok(ndt)
}

fn init_logger() {
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "info");
  }
  init_env_logger();
}

fn main() -> Result<()> {
  init_logger();

  info!("Application started.");

  let args = Args::parse();

  let input_reader: Box<dyn std::io::Read> = if let Some(input_path) = args.input {
    Box::new(File::open(input_path)?)
  } else {
    Box::new(stdin())
  };

  let output_writer: Box<dyn std::io::Write> = if let Some(output_path) = args.output {
    Box::new(File::create(output_path)?)
  } else {
    Box::new(stdout())
  };

  let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(
    DecodeReaderBytesBuilder::new()
      .encoding(Some(SHIFT_JIS))
      .build(BufReader::new(input_reader)),
  );

  let mut writer = csv::WriterBuilder::new()
    .has_headers(false)
    .from_writer(BufWriter::new(output_writer));

  let mut is_header = true;
  let mut prev_dt: Option<NaiveDateTime> = None;

  for result in reader.records() {
    let record = result?;
    let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

    if is_header {
      // ヘッダー行はそのまま出力
      let encoded_fields: Vec<Vec<u8>> = fields
        .iter()
        .map(|f| {
          let (enc, _, _) = SHIFT_JIS.encode(f);
          enc.into_owned()
        })
        .collect();
      let ref_fields: Vec<&[u8]> = encoded_fields.iter().map(|v| v.as_ref()).collect();
      writer.write_record(ref_fields)?;
      is_header = false;
      continue;
    }

    // 1列目が日時
    if let Some(datetime_str) = fields.get(0) {
      let dt = match conv_time(datetime_str) {
        Ok(d) => d,
        Err(e) => {
          warn!("Failed to parse datetime '{}': {}", datetime_str, e);
          continue; // この行はスキップ
        }
      };

      // 時系列チェック
      if let Some(prev) = prev_dt {
        if dt < prev {
          // 前の時刻より古い→除外してwarn
          warn!("Out-of-order record detected ({} < {}), skipping", dt, prev);
          continue;
        }
      }

      // 時系列OK、prev_dt更新
      prev_dt = Some(dt);

      let formatted = dt.format("%Y/%m/%d %H:%M:%S").to_string();
      fields[0] = formatted;
    } else {
      // 日付列がない場合→おかしいが、そのまま出力するかスキップするか
      // 要求にはないが、ここでは一応warnしてスキップ
      warn!("No datetime field found, skipping this record.");
      continue;
    }

    // 各フィールドをShiftJISエンコード
    let encoded_fields: Vec<Vec<u8>> = fields
      .iter()
      .map(|f| {
        let (enc, _, _) = SHIFT_JIS.encode(f);
        enc.into_owned()
      })
      .collect();
    let ref_fields: Vec<&[u8]> = encoded_fields.iter().map(|v| v.as_ref()).collect();
    writer.write_record(ref_fields)?;
  }

  writer.flush()?;
  Ok(())
}

/* async fn  *********************************************************************************************/

/* test for pri ******************************************************************************************/

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_conv_time() {
    let x = "11/25/24, 11:28:34 AM GMT+9";
    let dt = conv_time(x).unwrap();
    let expected = NaiveDate::from_ymd_opt(2024, 11, 25)
      .unwrap()
      .and_hms_opt(11, 28, 34)
      .unwrap();
    assert_eq!(dt, expected);
  }
}

/* test for pub ******************************************************************************************/
