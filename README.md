# multi-threaded-news-scraper
A multi-threaded news article scraper via Google Console API (Search) and headless chrome browser. Written in Rust.

This project was scrapped from a service project under the UP Center for Student Innovations for a client working on her master thesis.
The project was intended to scrape other news articles, but only Sun Star was implemented.

## Setup

Rust needs to be installed to run the scraper. The scraper will download the data from the website and save it to a file called `articles.csv`.

Be sure that RUST is installed in your system. If not, you can install it from [here](https://www.rust-lang.org/tools/install).
Rust is working if you can run `cargo --version` in your terminal.

Initialize web scraper. This also installs necessary dependencies which takes a while!

```bash
cargo install --path ./web_scraper --verbose
cargo build --path ./web_scraper --release
```

## Execute the web scraper. 
**This uses a lot of due to instantiation of web pages!**
Each webpage found will open a headless tab of Google Chrome. Be sure to have a lot of RAM.
Ensure that you have created an .env file with the following variables:
- SEARCH_API_KEY: can be obtained by 'Show key' on https://developers.google.com/custom-search/v1/overview/
- SEARCH_CX : can be obtained from Google's Custom search JSON API https://programmablesearchengine.google.com/controlpanel/create

An example of the .env file:
```
SEARCH_API_KEY=your_api_key
SEARCH_CX=your_cx
```
To run the scraper, run

```
cargo run --manifest-path ./web_scraper/Cargo.toml --release "TOPIC HERE" N_ARTICLES
```
If you get the following:

```
Request failed with status 429 Too Many Requests
```

It means that your API key has been used too many times. You can wait for a while or generate another API key. Google limitations :(

Google only allows 100 requests per month which is plentiful for basic scraping. A better way around here is to manually scrape the webpages using the headless browser.


