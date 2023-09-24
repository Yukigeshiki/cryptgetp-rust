#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::Client;

const COIN_API_URL: &str = "https://rest.coinapi.io/v1/exchangerate";

#[derive(Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
/// A just for fun CLI tool to fetch cryptocurrency prices written in Rust
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Fetches the price of a given cryptocurrency (--crypto) returned in a given fiat currency (--fiat)
    Fetch {
        #[arg(short, long)]
        /// The cryptocurrency you want to fetch the price for
        crypto: String,

        #[arg(short, long)]
        /// The fiat currency the price will be returned in
        fiat: String,

        #[arg(short, long)]
        /// The API key from https://www.coinapi.io/pricing?apikey
        key: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.cmd {
        Cmd::Fetch { crypto, fiat, key } => {
            let url = format!("{COIN_API_URL}/{crypto}/{fiat}");
            match get_data(Client::new(), url, key).await {
                Ok(data) => {
                    let s = format!(
                        "At the time {} the price of {} in {} was {}",
                        data.time, data.asset_id_base, data.asset_id_quote, data.rate
                    );
                    println!("{}", s.bright_cyan());
                }
                Err(e) => eprintln!("{} {e}", "error:".to_string().bright_red()),
            }
        }
    }
}

async fn get_data(client: Client, url: String, key: String) -> Result<CoinApiData, Error> {
    let res = client
        .get(url)
        .header("X-CoinAPI-Key", key)
        .send()
        .await
        .map_err(|_| Error::CoinApi)?;
    let status = res.status();
    if !status.is_success() {
        Err(Error::Response(status.as_u16()))?;
    }
    let text = res.text().await.map_err(|_| Error::CoinApi)?;
    serde_json::from_str(&text).map_err(|_| Error::CoinApi)
}

#[derive(serde::Deserialize)]
struct CoinApiData {
    time: String,
    asset_id_base: String,
    asset_id_quote: String,
    rate: f64,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("error fetching data from Coin API")]
    CoinApi,

    #[error("response from Coin API returned HTTP error code: {0}")]
    Response(u16),
}

#[cfg(test)]
mod tests {
    use super::get_data;
    use reqwest::Client;
    use wiremock::matchers::{any, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_data() {
        let mock_server = MockServer::start().await;
        let crypto = "BTC";
        let fiat = "USD";
        let p = format!("{crypto}/{fiat}");

        Mock::given(any())
            .and(path("/".to_owned() + &p))
            .and(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                    "time": "2017-08-09T14:31:18.3150000Z",
                    "asset_id_base": "BTC",
                    "asset_id_quote": "USD",
                    "rate": 26627.400434529947
                }"#,
                "application/json",
            ))
            .expect(1)
            .mount(&mock_server)
            .await;

        let data = get_data(
            Client::new(),
            format!("{}/{}", mock_server.uri(), p),
            "key".to_string(),
        )
        .await
        .expect("Failed to get data");

        assert_eq!(data.asset_id_base, crypto);
        assert_eq!(data.asset_id_quote, fiat);
        assert_eq!(data.rate, 26627.400434529947);
    }
}
