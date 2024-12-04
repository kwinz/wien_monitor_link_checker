use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://www.wien.gv.at/regierungsabkommen2020/regierungsmonitor/?kategorien=&umsetzungsstand=&page=10";

    let response: Result<reqwest::Response, Error> = reqwest::get(url).await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await?;

            println!("HTTP Status Code: {}", status);
            println!("Response Body:\n{}", body);
        }
        Err(err) => {
            eprintln!("Failed to make request: {}", err);
        }
    }

    Ok(())
}
