#![allow(unused)]
use std::fmt::{Debug, Display, Formatter};
use std::error::Error;
use std::num::ParseIntError;

// we want a custom Debug impl to expose the error stacktrace.
enum LargerAppError {
    Timeout,
    BadNews(Box<dyn std::error::Error>)
}

impl Debug for LargerAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => {
                write!(f, "{self}: timed out")
            },
            Self::BadNews(e) =>  {
                write!(f, "{self}\nCaused By:\n\t{e}");
                let mut err = e.source();
                while let Some(e) = err {
                    write!(f, "\nCaused By:\n\t{e}\n\t{e:?}");
                    err = e.source();
                }
                Ok(())
            }
        }
    }
}

impl Display for LargerAppError {
   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "A larger app error occurred.")
   }
}

#[derive(Debug)]
enum MiddleFunctionError {
    IoError(std::io::Error),
    Other(Box<dyn std::error::Error>)
}

impl Error for MiddleFunctionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MiddleFunctionError::IoError(e) => {
                Some(e)
            },
            MiddleFunctionError::Other(e) => {
                Some(e.as_ref())
            },
        }
    }
}

impl Display for MiddleFunctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "A middle function error occurred.") 
    }
}

// Would go in `LargerAppError`'s `Other` variant
// and it's `Error` trait impl of `source` is how we'd get at it's depths
// which just happen to paperclip themselves as a struct field and a `msg`.
// -> this isn't uniquely how `source` is supposed to work, in otherwords.
#[derive(Debug)]
struct DeepParsingError {
    msg: String,
    source: Box<dyn std::error::Error>
}

impl Display for DeepParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "A deep parsing error occurred:\n\t{0}",self.msg)
    }
}

impl Error for DeepParsingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

// Equivalent of doing a .map_err(|e| {})?
impl From<ParseIntError> for DeepParsingError {
   fn from(e: ParseIntError) -> Self {
      DeepParsingError {
          msg: "Error parsing value".to_string(),
          source: Box::new(e),
      } 
   } 
}

fn cut_up_ints(val: String) -> Result<i32, DeepParsingError> {
    Ok(
        val.parse::<i32>()?
    )
}

fn slice_n_dice(phones: Vec<String>) -> Result<Vec<i32>, MiddleFunctionError> {
    
    //let stuff = std::fs::read_to_string("goopy.txt").map_err(|e| {
    //    MiddleFunctionError::IoError(e)
    //})?;
    
    phones
        .into_iter()
        .map(|x| {
            cut_up_ints(x).map_err(|e| {
                MiddleFunctionError::Other(Box::new(e))
            })
        })
        .collect()
}

fn parse_phones(phones: Vec<String>) -> Result<Vec<i32>, LargerAppError> {
    slice_n_dice(phones).map_err(|e| {
        LargerAppError::BadNews(Box::new(e))
    })
}

#[tokio::main]
async fn main() {
    
    let phones = vec![
        "80047".to_string(),
        "1234567".to_string(),
        "99d547".to_string(),
    ];
    
    match parse_phones(phones) {
        Ok(vec)  => {
            println!("\n --- parsed phones: ---\n{:?}", vec);
        },
        Err(e) => {
            println!("\n --- main program ended with error traceback: ---\n{:?}", e);
        }
    }
}
