# slack-stream-json

Prints Slack's [Real Time Messaging](https://api.slack.com/rtm) (RTM) API streams to stdout.

## Usage

Set `SLACK_TOKEN` environment variable to a token for RTM API, for example obtained from [Legacy Tokens](https://api.slack.com/custom-integrations/legacy-tokens]) page. Once invoked, slack-stream-json prints RTM event JSON line by line.

    slack-stream-json 0.1.0

    USAGE:
        slack-stream-json [FLAGS]

    FLAGS:
        -f, --format-message          Resolve Slack message format, including mentions and links
        -h, --help                    Prints help information
        -i, --inflate-fields          Inflate "user", "channel" ID fields to corresponding JSON objects
        -p, --print-start-response    Print rtm.start response JSON before starting RTM stream
        -V, --version                 Prints version information

## Install

* Download binaries from [Releases](https://github.com/motemen/slack-stream-json/releases), or
* Clone this repository and install by `cargo install`.
