#![allow(unused)]

use clap::Parser;
use colored::{ColoredString, Colorize};
use reqwest::Client;

const COIN_API_URL: &str = "https://rest.coinapi.io/v1/exchangerate";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The cryptocurrency to fetch
    #[arg(short, long)]
    crypto: String,

    /// The fiat currency the price will be returned in
    #[arg(short, long)]
    fiat: String,

    /// The API key from https://www.coinapi.io/pricing?apikey
    #[arg(short, long)]
    key: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = format!("{}/{}/{}", COIN_API_URL, args.crypto, args.fiat);

    let client = Client::new();
    match get_data(client, url, args.key).await {
        Ok(data) => {
            let s = format!(
                "\nAt the time {} the price of {} in {} was {}\n",
                data.time, data.asset_id_quote, data.asset_id_base, data.rate
            );

            println!("{}", s.bright_cyan());
        }
        Err(e) => eprintln!("\n{}\n", e.to_string().bright_red()),
    }
}

async fn get_data(client: Client, url: String, key: String) -> Result<CoinApiData, Error> {
    let res = client
        .get(url)
        .header("X-CoinAPI-Key", key)
        .send()
        .await
        .map_err(|_| Error::CoinApi)?;
    // check status first
    let status = res.status();
    if !status.is_success() {
        Err(Error::ApiResponse(status.as_u16()))?;
    }
    let text = res.text().await.map_err(|_| Error::CoinApi)?;
    serde_json::from_str(&text).map_err(|_| Error::CoinApi)
}

#[derive(serde::Deserialize, Debug)]
struct CoinApiData {
    time: String,
    asset_id_base: String,
    asset_id_quote: String,
    rate: f64,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Response from Coin API returned error code: {0}")]
    ApiResponse(u16),

    #[error("Error fetching data from Coin API")]
    CoinApi,
}

#[cfg(test)]
mod tests {
    use super::get_data;
    use reqwest::Client;
    use wiremock::matchers::{any, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_data() {
        let mock_server = MockServer::start().await;

        Mock::given(any())
            .and(path("/BTC/USD"))
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
            format!("{}/{}", mock_server.uri(), "BTC/USD"),
            "key".to_string(),
        )
        .await
        .expect("Failed to get data");

        assert_eq!(data.asset_id_quote, "USD");
        assert_eq!(data.asset_id_base, "BTC");
        assert_eq!(data.rate, 26627.400434529947);
    }
}
