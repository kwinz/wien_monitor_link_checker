use itertools::Itertools;
use regex::Regex;
use reqwest::{Error, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://www.wien.gv.at/regierungsabkommen2020/regierungsmonitor/?kategorien=&umsetzungsstand=&page=10";
    let pattern = r"<h3>(.*)</h3>"; // Matches HTTP/HTTPS URLs
    let pattern2 = "<a href=\"(.*)\">(.*)</a>";

    let response: Result<reqwest::Response, Error> = reqwest::get(url).await;

    if let Ok(resp) = response {
        let status = resp.status();
        let body = resp.text().await?;

        println!("HTTP Status Code: {}", status);
        //println!("Response Body:\n{}", body);

        let re = Regex::new(pattern).unwrap();

        let sections: Vec<_> = re
            .find_iter(&body)
            .map(Some)
            .chain([None])
            .tuple_windows()
            .map(|(new_match, next)| {
                let name = new_match.unwrap().as_str();
                let start: usize = new_match.unwrap().start();
                let end = new_match.unwrap().end();

                println!("Found match {:?}", new_match);
                println!("Next match {:?}", next);

                new_match
            })
            //.map(|mat| (mat.as_str(), mat.end()))
            .collect();

        println!("Found matchs {:?}", sections);
    } else if let Err(err) = response {
        eprintln!("Failed to make request: {}", err);
    }

    Ok(())
}
