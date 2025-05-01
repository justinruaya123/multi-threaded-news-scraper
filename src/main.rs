use serde::Deserialize;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::env::args; 

use std::error::Error;
use serde_json;

use dotenv::dotenv;

use std::time::Instant;

mod scrapers;
mod interceptor;


#[derive(Deserialize, Debug)]
struct SearchResponse {
    items: Vec<Item>, 
}

#[derive(Deserialize, Debug)]
pub struct Item {
    link: String,
    #[serde(rename = "displayLink")]
    display_link: String
}


#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("SEARCH_API_KEY").expect("Error on .env file (or lack thereof): SEARCH_API_KEY can be obtained by 'Show key' on https://developers.google.com/custom-search/v1/overview");
    let cx = std::env::var("SEARCH_CX").expect("Error on .env file (or lack thereof): SEARCH_CX can be obtained from Google's Custom search JSON API https://programmablesearchengine.google.com/controlpanel/create");

    let args: Vec<String> = args().collect();
    if args.len() < 2 || args.len() > 3 {
        panic!("Usage: 'web_scraper <query> (num_articles || 50)'\n If you need multi-word queries, use double quotes: web_scraper \"multi word query\"");
    }
    let query = &args[1];
    let num_articles = args.get(2).unwrap_or(&"50".to_string()).parse::<usize>().unwrap();
    
    if query.is_empty() {
        panic!("Query cannot be empty");
    }

    println!("Scraping articles for query: {}", query);
    
    let now = Instant::now();
    let articles = search_articles(query, api_key, cx, num_articles).await.unwrap();
    println!("Fetched {} candidate articles...", articles.len());
    println!("Visiting webpages... ({:.2?})", now.elapsed());
    let treated_articles = scrapers::run_scraper(articles).await;
    println!("Found {} articles ({:.2?})", treated_articles.len(), now.elapsed());
    let mut file = OpenOptions::new().write(true).create(true).open("articles.csv").await.unwrap();
    for line in treated_articles{
        file.write_all(line.as_bytes()).await.unwrap();
        file.write_all(b"\n").await.unwrap();
    }
    println!("File saved as articles.csv ({:.2?})", now.elapsed());
}

async fn search_articles(query: &str, api_key: String, cx: String, count: usize) -> Result<Vec<Item>, Box<dyn Error + Send + Sync>> {

    let mut visit_queue = Vec::new();
    let mut tasks = Vec::new();

    for i in (1..count).step_by(10) {
        let url = format!(
            "https://www.googleapis.com/customsearch/v1?q={}&key={}&cx={}&fields=items(title,link,snippet,displayLink)&start={}",
            query, api_key, cx, i
        );

        let task = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let response = client.get(&url).send().await;
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        let body = response.text().await.unwrap();
                        let deserialized: Result<SearchResponse, _> = serde_json::from_str(&body);
                        match deserialized {
                            Ok(search_response) => search_response.items,
                            Err(_) => Vec::new(),
                        }
                    } else {
                        println!("Request failed with status {}", response.status());
                        Vec::new()
                    }
                }
                Err(err) => {
                    eprintln!("Request error: {}", err);
                    Vec::new()
                }
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let items = task.await.unwrap();
        if !items.is_empty() {
            visit_queue.extend(items);
        }
    }

    Ok(visit_queue)
}