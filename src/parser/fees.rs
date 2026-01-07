use crate::amount_serde::deserialize as amount_serde;
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("the file '{0}' does not exist")]
    FileNotFound(PathBuf),
    #[error("unable to store entry for account")]
    UnableToStoreEntry,
}

#[derive(Debug, serde::Deserialize)]
struct Entry {
    #[serde(rename = "Amount", deserialize_with = "amount_serde")]
    pub amount: i64,
    #[serde(rename = "User ID")]
    pub account_id: String,
    #[serde(rename = "User Email")]
    pub email: String,
}

#[derive(Debug)]
struct AccountFees {
    pub account_id: String,
    pub email: String,
    pub transaction_count: u32,
    pub total_fees: i64,
}

impl AccountFees {
    pub fn new(account_id: &str, email: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            email: email.to_string(),
            transaction_count: 0,
            total_fees: 0,
        }
    }

    pub const fn add_fee(&mut self, fee: i64) {
        self.transaction_count += 1;
        self.total_fees += fee;
    }

    pub const fn csv_header() -> &'static str {
        "account_id,email,transaction_count,total_fees_eur"
    }
}

impl std::fmt::Display for AccountFees {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{:.2}",
            self.account_id,
            self.email,
            self.transaction_count,
            self.total_fees as f64 / 100.0
        )
    }
}

/// Parse a Stripe fees CSV file located at the given path.
///
/// # Errors
///
/// File was not found or could not be read.
/// Unable to parse the CSV file.
/// Unable to create an Entry from a CSV line.
/// Output file could not be created or written to.
pub fn parse(file: PathBuf, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(Error::FileNotFound(file).into());
    }

    let output = output.unwrap_or_else(|| {
        let mut output_path = file.clone();
        output_path.set_file_name(format!(
            "{}_out.csv",
            file.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
        ));
        output_path
    });

    println!("parsing fees from file: {}", file.display());
    let mut csv_reader = csv::Reader::from_path(file)?;
    let mut statistics: HashMap<String, AccountFees> = HashMap::new();
    for result in csv_reader.deserialize() {
        let entry: Entry = result?;

        if !statistics.contains_key(&entry.account_id) {
            statistics.insert(
                entry.account_id.clone(),
                AccountFees::new(&entry.account_id, &entry.email),
            );
        }

        statistics
            .get_mut(&entry.account_id)
            .ok_or(Error::UnableToStoreEntry)?
            .add_fee(entry.amount);
    }

    if output.exists() {
        std::fs::remove_file(&output)?;
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("writing results to file: {}", output.display());
    let mut output_file = File::create(&output)?;
    output_file.write_all(AccountFees::csv_header().as_bytes())?;
    output_file.write_all(b"\n")?;
    for statistic in statistics.values() {
        output_file.write_all(statistic.to_string().as_bytes())?;
        output_file.write_all(b"\n")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    macro_rules! fees_test {
        ($name:ident, $data:expr, $($property:ident is $value:expr),+) => {
            #[test]
            fn $name() {
                let mut reader = csv::Reader::from_reader($data.as_bytes());
                let entry: Entry = reader.deserialize().next().expect("should have entry").expect("should be ok");

                $(
                    assert_eq!(entry.$property, $value);
                )*
            }
        };
        ($name:ident, $data:expr, is_error) => {
            #[test]
            fn $name() {
                #[allow(clippy::string_lit_as_bytes)]
                let mut reader = csv::Reader::from_reader($data.as_bytes());
                let entry: Result<Entry, _> = reader.deserialize().next().expect("should have entry");
                assert!(entry.is_err());
            }
        };
        ($name:ident, file, $data:expr, is_error) => {
            #[test]
            fn $name() {
                let mut temp_file = NamedTempFile::new().expect("should create temp file");
                let out_temp_file = NamedTempFile::new().expect("should create out temp file");
                writeln!(temp_file, "{}", $data).expect("should write to temp file");

                let result = parse(temp_file.path().to_path_buf(), Some(out_temp_file.path().to_path_buf()));
                assert!(result.is_err());
            }
        };
        ($name:ident, file, $data:expr, is_ok) => {
            #[test]
            fn $name() {
                let mut temp_file = NamedTempFile::new().expect("should create temp file");
                let out_temp_file = NamedTempFile::new().expect("should create out temp file");
                writeln!(temp_file, "{}", $data).expect("should write to temp file");

                let result = parse(temp_file.path().to_path_buf(), Some(out_temp_file.path().to_path_buf()));
                assert!(result.is_ok());
            }
        };
    }

    fees_test!(
        entry_deserialize_valid,
        "Amount,User ID,User Email\n\"0,25\",acct_123,user@example.com",
        amount is 25,
        account_id is "acct_123".to_string(),
        email is "user@example.com".to_string()
    );

    fees_test!(
        test_entry_deserialize_zero_amount,
        "Amount,User ID,User Email\n\"0,00\",acct_000,zero@example.com",
        amount is 0
    );

    fees_test!(
        test_entry_deserialize_missing_field_email,
        "Amount,User ID\n0,25,acct_123",
        is_error
    );

    fees_test!(
        test_entry_deserialize_invalid_amount,
        "Amount,User ID,User Email\ninvalid,acct_123,user@example.com",
        is_error
    );

    fees_test!(
        test_parse_valid_csv,
        file,
        "Amount,User ID,User Email\n\"0,25\",acct_123,user@example.com\n\"1,50\",acct_456,test@example.com",
        is_ok
    );

    fees_test!(
        test_parse_empty_csv,
        file,
        "Amount,User ID,User Email",
        is_ok
    );

    fees_test!(
        test_parse_invalid_csv_format,
        file,
        "Amount,User ID,User Email\ninvalid,acct_123,user@example.com",
        is_error
    );

    fees_test!(
        test_parse_real_sample_format,
        file,
        "id,Created (UTC),Amount,Amount Refunded,Currency,User ID,User Email,Application ID,Transaction ID\nfee_1ABC123XYZ456789DEF,2025-12-31 14:30,\"0,25\",\"0,00\",eur,acct_1TEST001ABC123XYZ,user1@example.com,ca_ABC123XYZ456789DEF,ch_3ABC123XYZ456789DEF",
        is_ok
    );

    #[test]
    fn test_parse_file_not_found() {
        let non_existent_path = PathBuf::from("/tmp/non_existent_file_12345.csv");
        let result = parse(non_existent_path, None);

        assert!(result.is_err());
        let err_msg = result.expect_err("should be error").to_string();
        assert!(err_msg.contains("does not exist"));
    }
}
