use atty::Stream;
use clap::Parser;
use error::ScrapeError;
use hyper::Client;
use hyper_tls::HttpsConnector;
use log::*;
use regex::Regex;
use scraper::Scraper;
use std::io::BufRead;
use std::process;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::printer::{Printer, TablePrinter, TextPrinter};

mod error;
mod printer;
mod scraper;

// NOTE(Wojciech): #[macro_use] extern crate xxx is unnecessary since I think Rust 2019, you can simply use
// <path_to_the_macro>; e.g. use log::info; to import the info macro.

const CHANNEL_BUFFER: usize = 500;

/// CLI tool to quickly scrape short snippets of text data from multiple HTTP sources
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Regex pattern to use in search including a target group to extract
    ///
    /// for example:
    ///     --pattern '<title>(.*)</title>'
    ///     --pattern '<meta name="og:site_name" content="([^"]+)"'
    #[clap(short, long)]
    pattern: String,

    /// Timeout (in seconds)
    #[clap(short, long, default_value_t = 5)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let regex =
        Regex::new(&args.pattern).expect("Error: pattern must be a valid regular expression!");

    let stdin = std::io::stdin();
    if atty::is(Stream::Stdin) {
        eprintln!("Error: stdin not redirected");
        process::exit(exitcode::IOERR);
    }

    // NOTE(Wojciech): Good use of a bounded channel to throttle scraping if you can scrape faster than you're able to save.
    type ChannelData = (String, Result<Option<String>, ScrapeError>);
    let (tx, mut rx) = mpsc::channel::<ChannelData>(CHANNEL_BUFFER);

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let scraper = Scraper::new(regex, args.timeout, client);

    for line in stdin.lock().lines() {
        let line = line.expect("Error: Could not read line from stdin");
        let tx = tx.clone();
        let scraper = scraper.clone();

        tokio::spawn(async move {
            let now = Instant::now();
            let result = {
                let uri = line.parse().map_err(|_| ScrapeError::InvalidURI)?;

                scraper.scrape(uri).await
            };
            // NOTE(Wojciech): So... this could be wrong because the await above could switch to other tasks while
            // downloading the website and then we don't know when it'll reschedule us so the end result doesn't
            // represent how long we've worked on this request rather how long we've worked on it plus how long we took
            // to get rescheduled.
            info!("Completed scrape for {} in {:?}", line, now.elapsed());
            tx.send((line.clone(), result))
                .await
                .map_err(|_| ScrapeError::SendError)
        });
    }

    // close the channel
    drop(tx);

    let mut output: Box<dyn Printer> = if atty::is(Stream::Stdout) {
        Box::new(TablePrinter::new())
    } else {
        Box::new(TextPrinter::new())
    };

    while let Some(res) = rx.recv().await {
        match res {
            (url, Ok(Some(matches))) => output.success(&url, &matches),
            (url, Ok(None)) => output.error(&url, ScrapeError::NoMatch.message()),
            (url, Err(e)) => output.error(&url, e.message()),
        };
    }

    output.finish();
}
