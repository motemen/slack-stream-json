use std::env;
use std::error::Error;

extern crate reqwest;
extern crate tungstenite;
extern crate slack_api;

// ref. https://slack-rs.github.io/slack-rs/src/slack/lib.rs.html#147-245

fn main() -> Result<(), Box<dyn Error>> {
    let token = match env::var("SLACK_TOKEN") {
        Ok(v) => v,
        Err(err) => panic!("Error {}", err),
    };
    let client = reqwest::Client::new();
    let start_response = slack_api::rtm::start(&client, &token, &Default::default())?;
    let start_url = start_response.url.as_ref().expect("start_url!!!!");
    let wss_url = reqwest::Url::parse(&start_url)?;
    let (mut websocket, _) = tungstenite::connect(wss_url)?;

    loop {
        let message = websocket.read_message()?;

        match message {
            tungstenite::Message::Text(text) => {
                println!("{}", text)
            }
            _ => {}
        }
    }

    Ok(())
}
