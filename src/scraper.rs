use super::ScrapeError;
use hyper::{body, Client, Uri};
use regex::Regex;
use std::time::Duration;
use tokio::time;

#[derive(Clone)]
pub struct Scraper<C> {
    regex: Regex,
    timeout: u64,
    client: Client<C>,
}

// NOTE(Wojciech): Rewrote the bounds a little, there's no needs to quote full paths to these traits, they are in the default prelude.
impl<C> Scraper<C>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    pub fn new(regex: Regex, timeout: u64, client: Client<C>) -> Self {
        Scraper {
            regex,
            client,
            timeout,
        }
    }

    pub async fn scrape(&self, uri: Uri) -> Result<Option<String>, ScrapeError> {
        let data = self.request(uri).await?;
        Ok(self.search(&data).map(|s| s.to_string()))
    }

    async fn request(&self, uri: Uri) -> Result<String, ScrapeError> {
        // NOTE(Wojciech): It's better to tell the user why the request failed. I tried calling https://google.com and
        // all I was told is REQUEST FAILED, that's it.

        let data =
            match time::timeout(Duration::from_secs(self.timeout), self.client.get(uri)).await {
                Ok(result) => result.map_err(|_| ScrapeError::RequestFailed),
                Err(_) => Err(ScrapeError::RequestTimeout),
            }?;

        // NOTE(Wojciech): The previous code will fail even if the request succeeded but is, let's say, a redirect or not a 200.
        if !data.status().is_success() {
            // warn!("Request failed: {:?}", data);

            return Err(ScrapeError::RequestFailed);
        }

        let bytes = body::to_bytes(data)
            .await
            .map_err(|_| ScrapeError::InvalidResponse)?;

        String::from_utf8(bytes.to_vec()).map_err(|_| ScrapeError::InvalidResponse)
    }

    // NOTE(Wojciech):
    // - This used to be async but if you look at it there're no blocking (in the async sense e.g. reading a file with
    //   async I/O) operations inside so it doesn't make sense to make this an async function. The tokio::spawn_blocking
    //   is just a way to run blocking (in the thread sense e.g. waiting on a mutex) functions in an asynchronous
    //   context, it's not needed here.
    // _ The clone on the regex is unnecessary. If you look at Regex::captures(), it takes a non-mutable &self so you
    //   can be sure that it won't be modified (I think this is why you cloned it?).
    // - You can replace the Strings with &str with a lifetime annotation to save on some allocations and make this a
    //   bit faster. This probably won't matter in practice unless you're scraping a lot... maybe millions of
    //   requests/strings. Actually it is completely unnecessary because the only thing you do with the result atm is
    //   .to_string() it :) I left the change in just for the record.
    // - I changed the behaviour a bit so when it captures mothing for capture group 1 it will also return nothing, not
    //   sure if this is what you wanted?
    //
    // Rule of thumb: if nothing inside a function needs to .await then het function probably shouldn't be async.
    fn search<'a>(&self, data: &'a str) -> Option<&'a str> {
        self.regex
            .captures(&data)
            .map(|m| m.get(1))
            .flatten() // NOTE(Wojciech): flattens Option<Option<T>> into Option<T>
            .map(|m| m.as_str())
    }
}

#[cfg(test)]
mod tests {
    // NOTE(Wojciech): Looks like tests were completely broken, missing some imports :)

    use crate::{error::ScrapeError, scraper::Scraper};
    use hyper::Client;
    use regex::Regex;
    use yup_hyper_mock::*;

    mock_connector!(MockResponses {
        "https://test.com" => "HTTP/1.1 200 OK\r\n\
                               \r\n\
                               High up ledges are out of reach, a jump to get there I'll now teach! Choose your spot with the greatest care, only one jump for bird and bear!"
        "https://bad.com" => "HTTP/1.1 400 Bad Request\r\n\
                              \r\n\
                              "

    });

    #[tokio::test]
    async fn test_scrape_extracts_matching_data() {
        let client = Client::builder().build::<_, hyper::Body>(MockResponses::default());

        let scraper = Scraper {
            client,
            regex: Regex::new(r"only one jump for (bird and bear)!").unwrap(),
            timeout: 5,
        };

        assert_eq!(
            scraper.scrape("https://test.com".parse().unwrap()).await,
            Ok(Some("bird and bear".to_string()))
        );
    }

    #[tokio::test]
    async fn test_scrape_returns_none_when_not_found() {
        let client = hyper::Client::builder().build::<_, hyper::Body>(MockResponses::default());

        let scraper = Scraper {
            client,
            regex: Regex::new(r"(/d)").unwrap(),
            timeout: 5,
        };

        assert_eq!(
            scraper.scrape("https://test.com".parse().unwrap()).await,
            Ok(None)
        );
    }

    #[tokio::test]
    async fn test_scrape_returns_error_when_request_fails() {
        let client = hyper::Client::builder().build::<_, hyper::Body>(MockResponses::default());

        let scraper = Scraper {
            client,
            regex: Regex::new(r"(/d)").unwrap(),
            timeout: 5,
        };

        assert_eq!(
            scraper.scrape("https://bad.com".parse().unwrap()).await,
            Err(ScrapeError::RequestFailed)
        );
    }
}
