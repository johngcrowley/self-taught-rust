use thiserror::Error as ThisError;
use serde::Deserialize;

#[derive(Debug, ThisError)]
enum AppError {
    #[error("took a while")]
    IoTimeout(#[from] std::io::Error),
    #[error("request errored")]
    NetworkTimeout(#[from] reqwest::Error),
}

impl std::fmt::Display for Users {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            write!(f, "{item:?}\n");
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Users(Vec<User>);

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    address: Address,
    phone: String,
    website: String,
    company: Company,
}

#[derive(Debug, Deserialize)]
struct Address {
    street: String,
    suite: String,
    city: String,
    zipcode: String,
    geo: Geo,
}

#[derive(Debug, Deserialize)]
struct Geo {
    lat: String,
    lng: String,
}

#[derive(Debug, Deserialize)]
struct Company {
    name: String,
    #[serde(default)]
    catch_phrase: String,
    bs: String,
}

async fn parse_users() -> Result<Users, AppError> {
    let url = "https://jsonplaceholder.typicode.com/users";
    let res = reqwest::get(url).await?;
    let users = res.json::<Users>().await?;
    Ok(users)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let users = parse_users().await?;
    println!("{users}");
    Ok(())
}
