use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::Deserialize;
use serde_json::Value as JSONValue;
use serde_json::Value::String as JSONString;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Inflate "user", "channel" ID fields to corresponding JSON objects
    #[structopt(short, long)]
    inflate_fields: bool,

    /// Resolve Slack message format, including mentions and links
    #[structopt(short, long)]
    format_message: bool,

    /// Print rtm.start response JSON before starting RTM stream
    #[structopt(short, long)]
    print_start_response: bool,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let token = std::env::var("SLACK_TOKEN").or(Err("SLACK_TOKEN not set"))?;

    let client = reqwest::Client::new();
    let rtm_response_text = client
        .get("https://slack.com/api/rtm.start")
        .query(&[("token", token)])
        .send()?
        .text()?;
    if opt.print_start_response {
        println!("{}", rtm_response_text);
    }
    let rtm_response: SlackRTMStartResponse = serde_json::from_str(&rtm_response_text)?;

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

    if let Some(err) = rtm_response.error {
        panic!(err)
    }

    let start_url = rtm_response.url.expect("Could not obtain RTM start_url");
    let wss_url = url::Url::parse(&start_url)?;
    let (mut websocket, _) = tungstenite::connect(wss_url)?;

    loop {
        let message = websocket.read_message()?;

        if let tungstenite::Message::Text(text) = message {
            // TODO: handle events like "channel_created" to update id_to_object
            // TODO: handle "goodbye" event
            let mut v: JSONValue = serde_json::from_str(&text)?;
            if opt.inflate_fields {
                // TODO: inflate "deeper" fields?
                inflate_field(&mut v, "user", &id_to_object);
                inflate_field(&mut v, "channel", &id_to_object);
            }
            if opt.format_message {
                if let JSONString(s) = &v["text"] {
                    v["text"] = JSONString(format_message(&s, &id_to_object))
                }
            }
            println!("{}", serde_json::to_string(&v)?)
        }
    }
}

// https://api.slack.com/docs/message-formatting#how_to_display_formatted_messages
fn format_message(message: &str, id_to_object: &HashMap<String, JSONValue>) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"&(?P<entity>amp|lt|gt);|<(?P<text>(?P<sign>[#@!]?)(?P<rest>.*?(?:\|(?P<title>.+?))?))>").unwrap();
    }

    String::from(RE.replace_all(message, |cap: &Captures| {
        if let Some(entity) = &cap.name("entity") {
            return String::from(match entity.as_str() {
                "amp" => "&",
                "lt" => "<",
                "gt" => ">",
                _ => unreachable!(),
            });
        }

        let text = &cap["text"];
        let sign = &cap["sign"];
        if let Some(title) = cap.name("title") {
            format!("{}{}", if sign == "!" { "" } else { sign }, title.as_str())
        } else if sign == "@" || sign == "#" {
            id_to_object.get(&cap["rest"]).map_or_else(
                || text.to_string(),
                |obj| format!("{}{}", sign, obj["name"].as_str().unwrap()),
            )
        } else if sign == "!" {
            format!("@{}", &cap["rest"])
        } else {
            text.to_string()
        }
    }))
}

#[test]
fn test_format_message() {
    use serde_json::json;

    let id_to_object: HashMap<String, JSONValue> = [
        ("U12345", json!({"name": "user12345"})),
        ("U99999", json!({"name": "user99999"})),
        ("C56789", json!({"name": "ch56789"})),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.clone()))
    .collect();

    assert_eq!(
        format_message("normal message", &id_to_object),
        "normal message",
    );

    assert_eq!(
        format_message("<@U12345>, <@U99999> and <@U00000>", &id_to_object),
        "@user12345, @user99999 and @U00000",
    );

    assert_eq!(
        format_message("<#C56789> <#C56789|ch>", &id_to_object),
        "#ch56789 #ch",
    );

    assert_eq!(
        format_message(
            "<https://www.example.com/|example> my site <https://www.example.com/>",
            &id_to_object
        ),
        "example my site https://www.example.com/",
    );

    assert_eq!(
        format_message(
            "<!subteam^S00000000|@subteam> <!channnel> <!here>",
            &id_to_object
        ),
        "@subteam @channnel @here",
    );

    assert_eq!(
        format_message("Foo &lt;!everyone&gt; bar <http://test.com>", &id_to_object),
        "Foo <!everyone> bar http://test.com",
    );
}

fn inflate_field(root: &mut JSONValue, key: &str, id_to_object: &HashMap<String, JSONValue>) {
    if let JSONString(id) = &root[key] {
        if let Some(object) = id_to_object.get(id) {
            root[key] = object.clone();
        }
    }
}
