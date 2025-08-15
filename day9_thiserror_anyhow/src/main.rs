#![allow(unused)]

use anyhow::{Result, Context};
use std::error::Error;

// we want a custom Debug impl to expose the error stacktrace.
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("A timeout occurred.")]
    Timeout,
    #[error("Generally, I've got some bad news.")]
    BadNews { #[from] source: SlicenDicingError }
}

#[derive(Debug, thiserror::Error)]
enum SlicenDicingError {
    #[error("An IO Error occurred.")]
    IoError(#[from] std::io::Error),
    #[error("A Conversion Error occurred.")]
    ConversionError { #[from] source: DeepParsingError } 
}

#[derive(Debug, thiserror::Error)]
#[error("A deep parsing error occurred: {msg}")]
struct DeepParsingError {
    msg: String,
    #[source]
    source: Box<dyn std::error::Error + Send + Sync>
}

fn cut_up_ints(val: String) -> Result<i32, DeepParsingError> {
    Ok(
        val.parse::<i32>().map_err(|e| DeepParsingError {
            msg: "Could not parse value {val}".to_string(),
            source: Box::new(e),
        })?
    )
}

fn slice_n_dice(phones: Vec<String>) -> Result<Vec<i32>, SlicenDicingError> {
    
    // TODO: `touch` this file to see the error in call stack
    //let stuff = std::fs::read_to_string("goopy.txt")?;
    
    phones
        .into_iter()
        .map(|x| {
            Ok(cut_up_ints(x)?)
        })
        .collect()
}

fn parse_phones(phones: Vec<String>) -> Result<Vec<i32>, AppError> {
    Ok(slice_n_dice(phones)?)
}

fn main() -> anyhow::Result<()> {
    
    let phones = vec![
        "80047".to_string(),
        "1234567".to_string(),
        "99d547".to_string(),
    ];
    
    let _ = parse_phones(phones)?;

    Ok(())
}
