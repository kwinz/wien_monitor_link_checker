use itertools::Itertools;
use regex::Regex;
use reqwest::{Error, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://www.wien.gv.at/regierungsabkommen2020/regierungsmonitor/?kategorien=&umsetzungsstand=&page=10";
    let uncompiled_h3_regex = r"<h3>(.*)</h3>"; // Matches HTTP/HTTPS URLs
    let uncompiled_link_regex = "<a href=\"(.*)\">(.*)</a>";

    let response: Result<reqwest::Response, Error> = reqwest::get(url).await;

    if let Ok(resp) = response {
        let status = resp.status();
        let body = resp.text().await?;

        println!("HTTP Status Code: {}", status);
        //println!("Response Body:\n{}", body);

        let h3_regex = Regex::new(uncompiled_h3_regex).unwrap();
        let link_regex = Regex::new(uncompiled_link_regex).unwrap();

        let mut h3_groups_iter = h3_regex.captures_iter(&body).peekable();

        let mut sections = vec![];

        while let Some(new_h3_match) = h3_groups_iter.next() {
            let h3 = new_h3_match.get(1).unwrap().as_str();
            let text_start = new_h3_match.get(0).unwrap().end();

            let text_end = if h3_groups_iter.peek().is_none() {
                body.len()
            } else {
                h3_groups_iter.peek().unwrap().get(0).unwrap().start()
            };

            sections.push((h3, text_start, text_end));
        }

        //println!("Found matchs {:?}", sections);

        for (h3, start, end) in sections {
            let all_text_of_h3 = &body[start..end];

            let links: Vec<_> = link_regex
                .captures_iter(all_text_of_h3)
                .map(|link_match| {
                    //get(0) is the full capture
                    let url = link_match.get(1).expect("url missing").as_str();
                    let name = link_match.get(2).expect("name missing").as_str();
                    format!("{name} : {url}")
                })
                .collect();

            println!("Match {} {:?}\n", h3, links);
        }
    } else if let Err(err) = response {
        eprintln!("Failed to make request: {}", err);
    }

    Ok(())
}
