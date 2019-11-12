extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate structopt;
extern crate tungstenite;

use serde::Deserialize;
use serde_json::Value as JSONValue;
use serde_json::Value::String as JSONString;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Inflate "user", "channel" ID fields to corresponding JSON objects
    #[structopt(short, long)]
    inflate: bool,
}

// https://api.slack.com/methods/rtm.start
#[derive(Clone, Debug, Deserialize)]
struct SlackRTMStartResponse {
    error: Option<String>,
    ok: bool,
    url: Option<String>,
    users: Option<Vec<JSONValue>>,
    channels: Option<Vec<JSONValue>>,
    groups: Option<Vec<JSONValue>>,
    mpims: Option<Vec<JSONValue>>,
    ims: Option<Vec<JSONValue>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let token = env::var("SLACK_TOKEN").or(Err("SLACK_TOKEN not set"))?;

    let client = reqwest::Client::new();
    let rtm_response: SlackRTMStartResponse = client
        .get("https://slack.com/api/rtm.start")
        .query(&[("token", token)])
        .send()?
        .json()?;

    let id_to_object = {
        let mut id_to_object: HashMap<String, JSONValue> = HashMap::new();

        let objects = vec![
            rtm_response.users,
            rtm_response.channels,
            rtm_response.groups,
            rtm_response.mpims,
            rtm_response.ims,
        ];

        for objects in &objects {
            if let Some(objects) = objects {
                for obj in objects {
                    if let JSONString(id) = &obj["id"] {
                        id_to_object.insert(id.to_string(), obj.clone());
                    }
                }
            }
        }

        id_to_object
    };

    let start_url = rtm_response.url.expect("Could not obtain RTM start_url");
    let wss_url = url::Url::parse(&start_url)?;
    let (mut websocket, _) = tungstenite::connect(wss_url)?;

    loop {
        let message = websocket.read_message()?;

        if let tungstenite::Message::Text(text) = message {
            let mut v: JSONValue = serde_json::from_str(&text)?;
            if opt.inflate {
                inflate_object(&mut v, "user", &id_to_object);
                inflate_object(&mut v, "channel", &id_to_object);
            }
            println!("{}", serde_json::to_string(&v).unwrap());
        }
    }
}

fn inflate_object(root: &mut JSONValue, key: &str, id_to_object: &HashMap<String, JSONValue>) {
    if let JSONString(id) = &root[key] {
        if let Some(object) = id_to_object.get(id) {
            root[key] = object.clone();
        }
    }
}
