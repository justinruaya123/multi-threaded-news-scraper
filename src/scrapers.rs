use std::{error::Error, sync::Arc};

use serde::Deserialize;

use headless_chrome::{Browser, LaunchOptions};
use serde_json::{self, Value};
use crate::{interceptor::ResourceInterceptor, Item};

#[derive(Debug, Deserialize)]
pub struct SunstarArticle {
    pub headline: String,
    #[serde(rename = "articleBody")]
    pub article_body: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    pub url: String
}

pub async fn run_scraper(items: Vec<Item>) -> Vec<String>{
    let mut handles = vec![];
    let mut return_articles: Vec<String> = Vec::new();
    let og_browser = Arc::new(Browser::new(LaunchOptions {
        sandbox: false,
        ..Default::default()
    }).unwrap());
    for (i, item) in items.into_iter().enumerate() {
        let browser = og_browser.clone();     
        let handle = tokio::spawn(async move {
            let tab = browser.new_tab().unwrap();
            tab.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3", 
                Some("en-US,en;q=0.9,fil-PH;q=0.8,fil;q=0.7"), 
                Some("Windows")).unwrap();
            let scraped_article = match item.display_link.as_str() {
                "www.sunstar.com.ph" => {
                    scrape_sunstar(&item.link, &tab).await
                },
                "www.philstar.com" => {
                    scrape_philstar(&item.link, &tab).await
                },
                "www.rappler.com" => {
                    scrape_rappler(&item.link, &tab).await
                },
                _ => {
                    println!("{}. Invalid URL: {}", i, item.display_link);
                    return "".to_string();
                }
            };
            println!("Handling url {}", item.link);
            match scraped_article {
                Ok(Some(article)) => {
                    return format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\"",
                        article.headline,
                        article.url,
                        article.date_modified,
                        article.article_body.replace("\"", "\"\"")
                    );
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return "".to_string();
                }
                _ => {
                    return "".to_string();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        if !result.is_empty() {
            return_articles.push(result.clone());
        }
        
    }
    return_articles
}



pub async fn scrape_sunstar(url: &str, tab: &Arc<headless_chrome::browser::Tab>) -> Result<Option<SunstarArticle>, Box<dyn Error + Send + Sync>> {
    let interceptor = Arc::new(ResourceInterceptor);

    tab.navigate_to(&url)?;
    // tab.wait_until_navigated()?;
    let _ = tab.enable_fetch(None, None);
    let _ = tab.enable_request_interception(interceptor);
    let script_content = tab.evaluate(
        r#"
        (function() {
            var scripts = document.querySelectorAll('script[type="application/ld+json"]');
            var jsonContents = [];
            scripts.forEach(function(script) {
                var content = script.innerText.trim();
                if (content.startsWith('{"headline":')) {
                    jsonContents.push(content);
                }
            });
            return JSON.stringify(jsonContents);
        })();
        "#,
        false,
    )?.value.unwrap().as_str().unwrap().to_string();
    println!("URL: {} and JSON: {}", url, script_content);

    let json_contents: Vec<String> = serde_json::from_str(&script_content)?;
    let _ = tab.close_with_unload();

    for json_content in json_contents.iter() {
        let parsed: Value = serde_json::from_str(json_content)?;
        if let (Some(headline), Some(article_body), Some(date_modified)) = (
            parsed.get("headline").and_then(|v| v.as_str()),
            parsed.get("articleBody").and_then(|v| v.as_str()),
            parsed.get("dateModified").and_then(|v| v.as_str()),
        ) {
            let article = SunstarArticle {
                headline: headline.to_string(),
                article_body: article_body.to_string(),
                date_modified: date_modified.to_string(),
                url: url.to_string()
            };
            return Ok(Some(article));
        }
    }
    Ok(None)
}

pub async fn scrape_philstar(url: &str, tab: &Arc<headless_chrome::browser::Tab>)  -> Result<Option<SunstarArticle>, Box<dyn Error + Send + Sync>> {
    let _ = tab;
    println!("Philstar scraper not implemented: {}", url);
    Ok(None)
}

pub async fn scrape_rappler(url: &str, tab: &Arc<headless_chrome::browser::Tab>)  -> Result<Option<SunstarArticle>, Box<dyn Error + Send + Sync>> {
    let _ = tab;
    println!("Rappler scraper not implemented: {}", url);
    Ok(None)
}