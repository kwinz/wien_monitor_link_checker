use build_html::{Html, HtmlContainer, HtmlPage};
use regex::Regex;
use reqwest::{Error, Response, StatusCode};
use std::collections::HashMap;
use std::time::Instant;
use tokio::time::timeout;

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum WebStatus {
    Result(StatusCode),
    Error,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let regierungsabkommen_url = "https://www.wien.gv.at/regierungsabkommen2020/regierungsmonitor/";
    let uncompiled_h3_regex = r"<h3>(.*)</h3>"; // Matches HTTP/HTTPS URLs
    let uncompiled_link_regex = "<a href=\"(.*)\">(.*)</a>";

    let mut url_to_usage_map: HashMap<String, Vec<String>> = HashMap::new();

    let start = Instant::now();

    //for i in 1..2 {
    for i in 1..10 {
        let page_url = if i == 1 {
            regierungsabkommen_url.to_string()
        } else {
            format!("{regierungsabkommen_url}seite-{i}")
        };
        println!("Requesting {}", page_url);
        let response: Result<reqwest::Response, Error> = reqwest::get(page_url).await;

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

                        //if url.to_string().contains("Grätzelinitiative") {
                        //    print!("{name} : {url}");
                        //}

                        url_to_usage_map
                            .entry(url.to_string())
                            .or_insert_with(Vec::new) // Ensure the value is a vector if the key is not present
                            .push(format!("{h3} : {name}"));

                        format!("{name} : {url}")
                    })
                    .collect();

                //println!("Match {} {:?}\n", h3, links);
            }
        } else if let Err(err) = response {
            eprintln!("Failed to make request: {}", err);
        }
    }

    let duration = start.elapsed();
    let elapsed_ms = duration.as_millis();
    println!("fetched and parsed 9 pages in {} ms", elapsed_ms);
    let start = Instant::now();

    println!("Found {} unique URLs\n", url_to_usage_map.keys().len());

    let mut status_to_url_map: HashMap<WebStatus, Vec<String>> = HashMap::new();

    let timeout_duration = tokio::time::Duration::from_millis(5000);

    for unique_url in url_to_usage_map.keys() {
        println!("{}\n", unique_url);

        let response = timeout(timeout_duration, reqwest::get(unique_url)).await;
        if let Ok(response) = response {
            if let Ok(response) = response {
                //println!("Status {}\n", response.status());

                status_to_url_map
                    .entry(WebStatus::Result(response.status()))
                    .or_insert_with(Vec::new) // Ensure the value is a vector if the key is not present
                    .push(unique_url.to_owned());
            } else {
                //network error
                status_to_url_map
                    .entry(WebStatus::Error)
                    .or_insert_with(Vec::new) // Ensure the value is a vector if the key is not present
                    .push(unique_url.to_owned());
            }
        } else {
            //timeout
            status_to_url_map
                .entry(WebStatus::Error)
                .or_insert_with(Vec::new) // Ensure the value is a vector if the key is not present
                .push(unique_url.to_owned());
        }
    }

    let duration = start.elapsed();
    let elapsed_ms = duration.as_millis();
    println!("checked liveliness in {} ms", elapsed_ms);

    //print!("{:?}", status_to_url_map);

    let mut page = HtmlPage::new().with_title("TITLE");

    for status in status_to_url_map.keys() {
        page = page.with_header(1, format!("{:?}", status));

        for url in status_to_url_map.get(status).unwrap() {
            page = page.with_header(2, url);

            for usage in url_to_usage_map.get(url).unwrap() {
                page = page.with_paragraph(format!("{}", usage));
            }
        }
    }

    let html_string = page.to_html_string();

    std::fs::write("output_iterative_with_builder.html", html_string)
        .expect("Unable to write file");

    Ok(())
}
