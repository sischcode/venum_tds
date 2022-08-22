use std::fmt::Debug;

use regex::Regex;
use venum::value::Value;

use crate::errors::{Result, SplitError, TransformErrors, VenumTdsError};

pub trait ValueSplit: Debug {
    fn split(&self, src: &Value) -> Result<(Value, Value)>;
}

pub trait ValueSplitN: Debug {
    fn split_n(&self, src: &Value) -> Result<Vec<Value>>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValueStringSeparatorCharSplit {
    pub sep_char: char,
    pub split_none: bool,
}

impl ValueSplit for ValueStringSeparatorCharSplit {
    fn split(&self, src: &Value) -> Result<(Value, Value)> {
        if src.is_some() {
            match src {
                Value::String(s) => {
                    let splitted: Vec<&str> = s.split(self.sep_char).collect();
                    if splitted.len() != 2 {
                        return Err(VenumTdsError::Transform(TransformErrors::Split(
                            SplitError::new(
                                format!(
                                    "expected 2 tokens as result of split, but got: {}",
                                    splitted.len()
                                ),
                                src.clone(),
                            ),
                        )));
                    };
                    Ok((
                        Value::from(String::from(splitted[0])),
                        Value::from(String::from(splitted[1])),
                    ))
                }
                _ => Err(VenumTdsError::Transform(TransformErrors::Split(
                    SplitError::new(
                        String::from("Not a Value::String. Can't split."),
                        src.clone(),
                    ),
                ))),
            }
        } else if self.split_none {
            Ok((Value::None, Value::None))
        } else {
            Err(VenumTdsError::Transform(TransformErrors::Split(
                SplitError::new(
                    String::from("Value is None, but split_none is false"),
                    src.clone(),
                ),
            )))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValueStringSeparatorCharSplitN {
    pub sep_char: char,
    pub split_none: bool,
    pub split_none_into_num_clones: Option<usize>,
}

impl ValueSplitN for ValueStringSeparatorCharSplitN {
    fn split_n(&self, src: &Value) -> Result<Vec<Value>> {
        if src.is_none() {
            match (&self.split_none, &self.split_none_into_num_clones) {
                (true, None) =>  Err(VenumTdsError::Transform(TransformErrors::Split(SplitError::new(
                    String::from("Value is Value::None, split_none is true, but split_none_into_num_clones is not set. Can't split into undefined number of targets!"),
                    src.clone(),
                )))),
                (false, _) => Err(VenumTdsError::Transform(TransformErrors::Split(SplitError::new(
                    String::from("Value is Value::None but split_none is false. Not allowed to split!",),
                    src.clone(),                    
                )))),
                (true, Some(num_targets)) => {
                    let mut v: Vec<Value> = Vec::with_capacity(*num_targets);
                    for _ in 1..=*num_targets {
                        v.push(Value::None);
                    }
                    Ok(v)
                }
            }
        } else {
            match src {
                Value::String(s) => {
                    let splitted: Vec<&str> = s.split(self.sep_char).collect(); // this will never return a length of 0, as it's implemented by rust!
                    match splitted.len() {
                        1 => Err(VenumTdsError::Transform(TransformErrors::Split(
                            SplitError::new(
                                String::from(
                                    "expected 2 (or more) tokens as result of split, but got: 1",
                                ),
                                src.clone(),
                            ),
                        ))),
                        _ => Ok(splitted
                            .into_iter()
                            .map(|v| Value::from(String::from(v)))
                            .collect()),
                    }
                }
                _ => Err(VenumTdsError::Transform(TransformErrors::Split(
                    SplitError::new(
                        String::from("Not a Value::String. Can't split."),
                        src.clone(),
                    ),
                ))),
            }
        }
    }
}

#[derive(Debug)]
pub struct ValueStringRegexPairSplit {
    pub re: Regex,
    pub split_none: bool,
}

impl ValueStringRegexPairSplit {
    pub fn new(regex_pattern: String, split_none: bool) -> Result<Self> {
        let re = Regex::new(regex_pattern.as_str()).map_err(|e| {
            let mut err_msg = format!("{}", e);
            err_msg.push_str(" (RegexPairSplitter, ERROR_ON_REGEX_COMPILE)");
            VenumTdsError::Transform(TransformErrors::Generic { msg: err_msg })
        })?;
        Ok(ValueStringRegexPairSplit { re, split_none })
    }
}

impl ValueSplit for ValueStringRegexPairSplit {
    fn split(&self, src: &Value) -> Result<(Value, Value)> {
        if src.is_some() {
            match src {
                Value::String(s) => {
                    let caps = self.re.captures(s).ok_or_else(|| {
                        VenumTdsError::Transform(TransformErrors::Split(
                            SplitError::new_with_details(
                                String::from("No captures, but we need exactly two."),
                                src.clone(),
                                Some(format!("regex: {}", self.re.as_str())),
                            ),
                        ))
                    })?;
                    if caps.len() == 3 {
                        let token_match_1 = caps.get(1).unwrap().as_str(); // checked above, via .len()
                        let token_match_2 = caps.get(2).unwrap().as_str(); // checked above, via .len()
                        Ok((
                            Value::String(String::from(token_match_1)),
                            Value::String(String::from(token_match_2)),
                        ))
                    } else {
                        Err(VenumTdsError::Transform(TransformErrors::Split(
                            SplitError::new_with_details(
                                format!(
                                    "{} capture group(s), but we need exactly two.",
                                    caps.len() - 1
                                ),
                                src.clone(),
                                Some(String::from(self.re.as_str())),
                            ),
                        )))
                    }
                }
                _ => Err(VenumTdsError::Transform(TransformErrors::Split(
                    SplitError::new(
                        String::from("Not a Value::String. Can't split."),
                        src.clone(),
                    ),
                ))),
            }
        } else if self.split_none {
            Ok((Value::None, Value::None))
        } else {
            Err(VenumTdsError::Transform(TransformErrors::Split(
                SplitError::new(
                    String::from("Value is None, but split_none is false"),
                    src.clone(),
                ),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_seperator_char() {
        let sep = ValueStringSeparatorCharSplit {
            sep_char: ' ',
            split_none: true,
        };
        let data = Value::from("foo bar".to_string());
        let split_res = sep.split(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(Value::from("foo".to_string()), split_vals.0);
        assert_eq!(Value::from("bar".to_string()), split_vals.1);
    }

    #[test]
    fn split_by_seperator_char_none() {
        let sep = ValueStringSeparatorCharSplit {
            sep_char: ' ',
            split_none: true,
        };
        let data = Value::None;
        let split_res = sep.split(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(Value::None, split_vals.0);
        assert_eq!(Value::None, split_vals.1);
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"expected 2 tokens as result of split, but got: 3\", src_val: String(\"foo bar baz\"), details: None }))"
    )]
    fn split_by_seperator_char_err() {
        let sep = ValueStringSeparatorCharSplit {
            sep_char: ' ',
            split_none: true,
        };
        let data = Value::from("foo bar baz".to_string());
        sep.split(&data).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"expected 2 tokens as result of split, but got: 1\", src_val: String(\"foo\"), details: None })"
    )]
    fn split_by_seperator_char_err2() {
        let sep = ValueStringSeparatorCharSplit {
            sep_char: ' ',
            split_none: true,
        };
        let data = Value::from("foo".to_string());
        sep.split(&data).unwrap();
    }

    #[test]
    fn split_n_by_seperator_char() {
        let sep = ValueStringSeparatorCharSplitN {
            sep_char: ' ',
            split_none: false,
            split_none_into_num_clones: None,
        };
        let data = Value::from("foo bar baz".to_string());
        let split_res = sep.split_n(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(&Value::from("foo".to_string()), split_vals.get(0).unwrap());
        assert_eq!(&Value::from("bar".to_string()), split_vals.get(1).unwrap());
        assert_eq!(
            &Value::from("baz".to_string()),
            split_vals.get(2).unwrap()
        );
    }

    #[test]
    fn split_n_by_seperator_char_none() {
        let sep = ValueStringSeparatorCharSplitN {
            sep_char: ' ',
            split_none: true,
            split_none_into_num_clones: Some(3),
        };
        let data = Value::None;
        let split_res = sep.split_n(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(&Value::None, split_vals.get(0).unwrap());
        assert_eq!(&Value::None, split_vals.get(1).unwrap());
        assert_eq!(&Value::None, split_vals.get(2).unwrap());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is Value::None, split_none is true, but split_none_into_num_clones is not set. Can't split into undefined number of targets!\", src_val: None, details: None })"
    )]
    fn split_n_by_seperator_char_none_err_config() {
        let sep = ValueStringSeparatorCharSplitN {
            sep_char: ' ',
            split_none: true,
            split_none_into_num_clones: None,
        };
        let data = Value::None;
        sep.split_n(&data).unwrap();
    }

    #[test]
    fn split_by_regex_pair() {
        let sep_res =
            ValueStringRegexPairSplit::new("(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(), true);
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Value::from("1.12 2.23".to_string());
        let split_res = sep.split(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(Value::from("1.12".to_string()), split_vals.0);
        assert_eq!(Value::from("2.23".to_string()), split_vals.1);
    }

    #[test]
    #[should_panic(expected = "Split(SplitError { msg: \"No captures, but we need exactly two.\"")]
    fn split_by_regex_err_no_captures() {
        let sep_res = ValueStringRegexPairSplit::new(
            "(\\d+\\.\\d+).*(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
            true,
        );
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Value::from("1.12 2.23".to_string());
        sep.split(&data).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"1 capture group(s), but we need exactly two.\""
    )]
    fn split_by_regex_err_too_few_capture_groups() {
        let sep_res = ValueStringRegexPairSplit::new("(\\d+\\.\\d+)".to_string(), true);
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Value::from("1.12 2.23".to_string());
        sep.split(&data).unwrap();
    }

    #[test]
    #[should_panic(expected = "Transform(Generic { msg: \"regex parse error")]
    fn split_by_regex_pair_illegal_regex() {
       ValueStringRegexPairSplit::new("FWPUJWDJW/)!(!()?))".to_string(), true).unwrap();        
    }
}
