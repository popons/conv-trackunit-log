/* use  **************************************************************************************************/

use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime};
use csv;
use encoding_rs::SHIFT_JIS;
use encoding_rs_io::DecodeReaderBytesBuilder;
use encoding_rs_io::EncodingWriter;
use env_logger::init as init_logger;
use log::info;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/* mod  **************************************************************************************************/

/* type alias  *******************************************************************************************/

/* global const  *****************************************************************************************/

/* trait  ************************************************************************************************/

/* enum  *************************************************************************************************/

/* struct  ***********************************************************************************************/

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
                            // タイムゾーン "GMT+9" は今回は無視

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

fn main() -> Result<()> {
  init_logger();
  info!("Application started.");

  let input_path = "input.csv";
  let output_path = "output.csv";

  let input_file = File::open(input_path)?;
  let output_file = File::create(output_path)?;

  let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(
    DecodeReaderBytesBuilder::new()
      .encoding(Some(SHIFT_JIS))
      .build(BufReader::new(input_file)),
  );

  let enc_writer = EncodingWriter::new(SHIFT_JIS, BufWriter::new(output_file));
  let mut writer = csv::WriterBuilder::new()
    .has_headers(false)
    .from_writer(enc_writer);

  let mut is_header = true;
  for result in reader.records() {
    let record = result?;
    let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

    if is_header {
      // ヘッダー行はそのまま出力
      writer.write_record(&fields)?;
      is_header = false;
      continue;
    }

    // 1列目が日時
    if let Some(datetime_str) = fields.get(0) {
      let dt = conv_time(datetime_str)?;
      // Excelで扱いやすい形式: "YYYY/MM/DD HH:MM:SS"
      let formatted = dt.format("%Y/%m/%d %H:%M:%S").to_string();
      fields[0] = formatted;
    }

    writer.write_record(&fields)?;
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
