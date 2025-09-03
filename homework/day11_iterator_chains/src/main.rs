use std::collections::HashMap;

/*
 * `map` consumes `self` that it is called upon, so it's __`map`'s__ return type of `Some()` or `Ok()`
 * 
 * it's a `match` statement under the hood, whose arm `=> Some(fn(t))`
 *
 * `and_then`'s `match` statement arm is `=> fn(t)`
 *
 * The type signiature for `and_then` shows the closure and function return type are the same
 * so the `fn(t)` therein is fallible, returning the `Some()` or `Ok()` (or `None`, etc) we expect
 *
 * The type signature for `map` shows the closure returns just the extracted value, where `map`'s
 * function return type wraps it with the `Some()` or `Ok()`.
 *
 * You always get confused because it seems like `and_then`'s closure + function both returning the
 * wrapper type seems like it would "double up" or nest the returned value, like `map` does when
 * you call `.map` on a fallible operation like `parse`.
 *
 */

#[derive(Debug, PartialEq)]
pub enum Command {
    Set(String, i32),
    Add(String, i32),
    Del(String),
}
#[tokio::main]
async fn main() {
    
    let inputs_1 = vec![
        "alex,500",
        "jordan,950",
        "invalid_line",    // skip
        ",700",            // skip, empty ID
        "casey,-300",      // skip, not a u32
        "morgan,1200",
    ];

    parser_1(inputs_1);
    
    let inputs_2 = vec![
        "set:id:100",
        "add:count:5",
        "del:user_session",
        "set:mode:debug",   // Should be skipped (value is not i32)
        "UNKNOWN:COMMAND",  // Should be skipped (invalid type)
        "add:retries:",     // Should be skipped (empty value)
        "del:",             // Should be skipped (empty key)
        "set:volume:99",
        "add:points:25",
        "del:user_session:extra", // Should be skipped (too many parts for del)
    ];

    parser_2(inputs_2);

    // let this represent a data stream. first and second lines are a success
    let log_data: &str = "INFO;request_id=a3f-9j1;method=GET;path=/api/v1/users;status=200;latency_ms=125\n\
                      INFO;request_id=457-d1;method=GET;path=/api/v1/users;status=200;latency_ms=574\n\
                      ERROR;request_id=b4g-0k2;method=POST;path=/api/v1/login;error=AuthFailure\n\
                      WARN;service=auth-service;message=High CPU usage\n\
                      BRONCO;411\n\
                      INFO;request_id=a3f-9j1;method=BOOP;path=/api/v1/users;status=450;latency_ms=125";
    
    parser_3(log_data).await;
    
}

/* Task 3:

Calculate avg latency for all 200 HTTP GET requests as f64.

Note we'll use `.fold()` and no elegant `match` on slices, because `match` needs a concrete size at compile time,
while iterators are lazily evaluated. This example shows a zero-copy strategy (no Vec allocation for the match or summation),
while still striving to be idiomatic.

*/

async fn parser_3(log_data: &str) -> Option<f64> {
    
    let (count, sum) = log_data
        .lines()
        // retain only non-`None`, and also modify what we retain, both based on `map`
        .filter_map(|line| {
            
            if !line.starts_with("INFO") {
                return None
            }
            
            let mut method = false;
            let mut status = false; 
            let mut latency = None;
            
            for val in line.split(";") {

                // no matter order
                match val {
                    "method=GET" => method = true,
                    "status=200" => status = true,
                    _ => {
                        if let Some(ms) = val.strip_prefix("latency_ms=") {
                            latency = ms.trim().parse::<f64>().ok();
                        }
                    }
                }
            }
            
            if method && status {
                latency
            } else {
                None
            }
        })
        .fold((0, 0.0), |(total_gets, total_latency), latency| {
            (total_gets + 1, total_latency + latency)
        });
    
    // prevent division by zero
    if count > 0 {
        
        println!("avg latency for {} requests: {} ms", 
            count, 
            sum / count as f64
        );
        Some(sum / count as f64)
    } else {
        None
    }
}


/* Task 1: 
 *
 * Parse inputs_1 into a map of name and score
 */
fn parser_1(v: Vec<&str>) {
   let parsed_map: HashMap<String, u32> = v
       // we can do everything without moving ownership.
       .iter()
       .filter_map(|line| {
          // `None` if no delimiter
          line
              .split_once(",")
              // closure is is fallible, so it's return type is return type of call stack
              // unlike `map`, which keeps its own return type (`Option`/`Result`), so if you
              // chain `map()` for fallible operations, you'll get nested `Some(Some())`s.
              .and_then(|(k,v)| {
                   if k.is_empty() {
                       return None
                   } 
                   // so (k,v) are inside the `Some()` return by `split_once`
                   v.parse::<u32>()
                        // `parse` is fallible, use `ok` to convert the `Result` to `Some`
                        // and `None` shortcircuts (missing our `filter_map`)
                       .ok()
                       // good example, we want to return stuff but retain the `Some()`
                       // we got from `ok`, and we can still reference `k` here, 
                       // returning a `Some()` that `filer_map` accepts and a tuple that
                       // can become a `HashMap`
                       .map(|parsed| {
                            (k.to_string(), parsed)      
                       })
              })
       })
       .collect();

   for (k, v) in parsed_map.iter() {
       println!("key: {k}\n\tval: {v}");
   }
}

/* Task 2:
 *
 * Parse inputs_2 into the CLI enum
 */
fn parser_2(v: Vec<&str>) {
    // just let them know what they picked 
    let cmds: Vec<Command> = v
        .iter()
        .filter_map(|line| {
            let cmd_vec: Vec<&str> = line.split(":").collect();
            match cmd_vec.as_slice() {

                // Add|Set(String, i32),
                [cmd @ ("set"|"add"), x, y] if !x.is_empty() => {
                    if let Ok(parsed) = y.parse::<i32>() {
                        if *cmd == "set" {
                            Some(Command::Set(x.to_string(), parsed))
                        } else {
                            Some(Command::Add(x.to_string(), parsed))
                        }
                    } else {
                        None
                    }
                },
                // Del(String),
                [cmd @ "del", x] => {
                    if x.is_empty() {
                        return None
                    }
                    Some(Command::Del(x.to_string()))
                },
                _ => None
            }
        })
        .collect();
    for command in cmds {
        println!("command: {command:?}");
    }
}
