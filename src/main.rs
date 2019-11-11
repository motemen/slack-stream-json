use std::env;
use std::error::Error;

extern crate slack_api;
extern crate tungstenite;

// ref. https://slack-rs.github.io/slack-rs/src/slack/lib.rs.html#147-245

fn main() -> Result<(), Box<dyn Error>> {
    let token = env::var("SLACK_TOKEN").or(Err("SLACK_TOKEN not set"))?;
    let client = slack_api::requests::default_client().unwrap();
    let start_response = slack_api::rtm::start(&client, &token, &Default::default())?;
    let start_url = start_response.url.expect("Could not obtain RTM start_url");
    let wss_url = url::Url::parse(&start_url)?;
    let (mut websocket, _) = tungstenite::connect(wss_url)?;

    loop {
        let message = websocket.read_message()?;

        match message {
            tungstenite::Message::Text(text) => println!("{}", text),
            _ => {}
        }
    }
}
