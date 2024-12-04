use regex::Regex;
use reqwest::{Error, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://www.wien.gv.at/regierungsabkommen2020/regierungsmonitor/?kategorien=&umsetzungsstand=&page=10";
    let pattern = r"<h3>.*</h3>"; // Matches HTTP/HTTPS URLs

    let response: Result<reqwest::Response, Error> = reqwest::get(url).await;

    if let Ok(resp) = response {
        let status = resp.status();
        let body = resp.text().await?;

        println!("HTTP Status Code: {}", status);
        //println!("Response Body:\n{}", body);

        let re = Regex::new(pattern).unwrap();

        for cap in re.find_iter(&body) {
            println!("Found match: {}, {:?}", cap.as_str(), cap);
        }
    } else if let Err(err) = response {
        eprintln!("Failed to make request: {}", err);
    }

    Ok(())
}
