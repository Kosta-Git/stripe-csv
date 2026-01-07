/// Parses amounts in Stripe CSV to eurocents.
///
/// Accepted format: "xxx,xx" or "xxx.xx"
///
/// # Errors
///
/// The amount is not in the specified format.
#[allow(clippy::cast_possible_truncation)]
pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    let normalized = s.replace(',', ".");
    let value = normalized
        .parse::<f64>()
        .map_err(serde::de::Error::custom)?;

    Ok((value * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct TestAmount {
        #[serde(deserialize_with = "deserialize")]
        amount: i64,
    }

    macro_rules! amount_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let csv_data = format!("amount\n\"{}\"", $input);
                let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
                let result: TestAmount = reader.deserialize().next().expect("should be ok").expect("should deserialize");
                assert_eq!(result.amount, $expected);
            }
        };
    }

    macro_rules! amount_error_test {
        ($name:ident, $input:expr) => {
            #[test]
            fn $name() {
                let csv_data = format!("amount\n\"{}\"", $input);
                let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
                let result: Result<TestAmount, _> = reader.deserialize().next().expect("should be ok");
                assert!(result.is_err());
            }
        };
    }

    amount_test!(test_period_decimal, "1.50", 150);
    amount_test!(test_comma_decimal, "1,50", 150);
    amount_test!(test_zero_amount, "0.00", 0);
    amount_test!(test_zero_amount_comma, "0,00", 0);
    amount_test!(test_large_amount, "123.45", 12345);
    amount_test!(test_large_amount_comma, "123,45", 12345);
    amount_test!(test_three_decimal_places, "1.234", 123);
    amount_test!(test_three_decimal_places_comma, "1,234", 123);
    amount_test!(test_rounding_up, "0.126", 13);
    amount_test!(test_rounding_up_comma, "0,126", 13);
    amount_test!(test_rounding_down, "0.124", 12);
    amount_test!(test_rounding_down_comma, "0,124", 12);
    amount_test!(test_integer_amount, "42", 4200);
    amount_test!(test_single_decimal, "1.5", 150);
    amount_test!(test_single_decimal_comma, "1,5", 150);

    amount_error_test!(test_invalid_format, "invalid");
    amount_error_test!(test_empty_string, "");
}
