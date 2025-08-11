#![allow(unused)]

//#[derive(Debug)]
pub enum AppError {
   NetworkError,
   IoError(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
/*
 *
 * With just `#[derive(Debug)]`, it would resemble:
 * ```
 * IoError(Os { code: 2, kind: NotFound, message: "No such file or directory" }) 
 * ```
 *
 * With the below, custom implementation, do the unraveling, without needing to 
 * impl Error's `.source()` method. A `match` statement will do.
 *
 * I thnk if `AppError` were `DeepAppError`, it would be a struct with a `source` field,
 * that we could paperclip the IO error to, then give `DeepAppError` an `impl Error`,
 * so the outer `AppError` could call that variant, that variant being its own struct.
 * 
 */
impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError => {
                write!(f, "{self}")
            },
            // the `{}` signifies "use the Dispaly message of the underlying message"
            Self::IoError(e) => {
                write!(f, "{self} \n Caused by: \n\t {e}")
            }
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError => {
                write!(f, "NetworkError occurred")
            },
            Self::IoError(e) => {
                write!(f, "IoError occurred")
            }
        }
    }
}

fn erroneously_read_file() -> Result<(), AppError> {
    let _ = std::fs::read_to_string("gooby-gop.txt")?;
    Ok(())
}

fn main() {
    match erroneously_read_file() {
       Ok(_) => {
           println!("yay, you will never reach me");
       },
       Err(e) =>  {
           // debug
           println!("{:?}", e);
       }
    }
}
