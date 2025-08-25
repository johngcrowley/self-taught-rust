#![allow(unused)]
use std::fmt::{Display, Debug, Formatter};
use std::error::Error;

enum MyError {
    AhShit(String),
    Ooopsie(String),
    Bahhhhh(MyNestedError),
}

// This is the key to get the chain going from Anyhow
impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Bahhhhh(e) => Some(e), // The nested error is the source
            _ => None
        }
    }
}

// If you `{e:?}` (call debug on `MyError` in `main`) you'd get our chain here
impl Debug for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // say the enum variant in `stdout`
        write!(f, "{self}");
        match self {
            MyError::Bahhhhh(e) => {
                let mut source = e.source();
                while let Some(s) = source {
                   write!(f, "Caused by:\n\t{s}");
                   source = s.source();
                       
                }
                
            },
            _ => {} 
        }
        Ok(())
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "There was an Error: ") 
    }
}

impl From<MyNestedError> for MyError {
    fn from(e: MyNestedError) -> Self {
        MyError::Bahhhhh(e)
    }
}

struct MyNestedError {
    msg: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl Debug for MyNestedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}") 
    }
}

impl Display for MyNestedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "There was a nested error:") 
    }
}

impl Error for MyNestedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(source) = &self.source {
            return Some(source.as_ref()) 
        }
        None
    }
}
/* 
        
   `filter_map` here will gracefully skip failed attempts
    so i'm using `map` to force me to handle the bad `parse`

    `filter_map` impls `Iterator`  trait on its `FilterMap` struct, i.e. its return type
   in that impl, it unwraps the `Option` it expects from the closure it calls

    `map` returns `Map` which is just a regular-ass Iterator impl that will take an `Option<B>`
   of closure-returned values and pass that on, rather than unpacking it, so `collect` will choke
       
*/

fn my_nested_function(users: Vec<&str>) -> Result<Vec<u32>, MyError> {
   let numbers = users
       .iter()
       // no option, no result, just does stuff and returns type from closure
       .map(|user| {
                // returns Some((x,y))
                let (_, y) = user.split_once(",").ok_or_else(|| MyError::Bahhhhh(
                            MyNestedError {
                                msg: "need a value to parse!".to_string(),
                                source: None,
                            })
                )?;
                
                let parsed = y.parse::<u32>().map_err(|e|
                        MyError::Bahhhhh(
                           MyNestedError {
                               msg: "need a value to parse!".to_string(),
                               source: Some(Box::new(e)),
                }))?;

                Ok(parsed)
                    
       }).collect::<Result<Vec<u32>, MyError>>();

   numbers
    
}

fn main() -> anyhow::Result<()> {
    
    let users = vec![
        "alex,500",
        "jordan,950",
        "casey,-300",
    ];

    let v = Some(my_nested_function(users));
    
    println!("{:?}", v);
    
    let gracefully_unwrapped = v
        // change our Some() wrapper to return "oopsie!" in case `v` were `None`
        .ok_or_else(|| MyError::Ooopsie("oopsie!".to_string()))
        // pluck out the nested Result
        .and_then(|x| x)
        // bubble it up!
        ?;
    
    Ok(())

}


