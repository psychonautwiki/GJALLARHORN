#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate regex;
extern crate tokio_core;

extern crate kafka;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

#[derive(Serialize, Deserialize)]
struct RedditPostData {
  subreddit_id: Option<String>,
  approved_at_utc: Option<String>,
  edited: bool,
  mod_reason_by: Option<String>,
  banned_by: Option<String>,
  author_flair_type: Option<String>,
  removal_reason: Option<String>,
  link_id: Option<String>,
  author_flair_template_id: Option<String>,
  likes: Option<String>,
  replies: Option<String>,
  saved: bool,
  id: Option<String>,
  banned_at_utc: Option<String>,
  mod_reason_title: Option<String>,
  gilded: i64,
  archived: bool,
  no_follow: bool,
  author: Option<String>,
  num_comments: Option<i64>,
  can_mod_post: bool,
  send_replies: bool,
  parent_id: Option<String>,
  score: i64,
  author_fullname: Option<String>,
  over_18: bool,
  approved_by: Option<String>,
  mod_note: Option<String>,
  collapsed: bool,
  body: Option<String>,
  link_title: Option<String>,
  author_flair_css_class: Option<String>,
  name: Option<String>,
  downs: i64,
  is_submitter: bool,
  body_html: Option<String>,
  gildings: Gildings,
  collapsed_reason: Option<String>,
  distinguished: Option<String>,
  stickied: bool,
  can_gild: bool,
  subreddit: Option<String>,
  author_flair_text_color: Option<String>,
  score_hidden: bool,
  permalink: Option<String>,
  num_reports: Option<String>,
  link_permalink: Option<String>,
  report_reasons: Option<String>,
  link_author: Option<String>,
  author_flair_text: Option<String>,
  link_url: Option<String>,
  created: i64,
  created_utc: i64,
  subreddit_name_prefixed: Option<String>,
  controversiality: i64,
  author_flair_background_color: Option<String>,
  quarantine: bool,
  subreddit_type: Option<String>,
  ups: i64,
}

#[derive(Serialize, Deserialize)]
struct Gildings {
  gid_1: i64,
  gid_2: i64,
  gid_3: i64,
}

#[derive(Serialize, Deserialize)]
struct RedditPost {
  kind: String,
  data: RedditPostData,
}

struct ConfiguredApi {
    api: telegram_bot::Api,
    core: tokio_core::reactor::Core,

    channel_id: i64,
    name: String,
    parse_mode: telegram_bot::types::ParseMode,
}

impl ConfiguredApi {
    fn new(name: &str, parse_mode: telegram_bot::types::ParseMode) -> ConfiguredApi {
        let telegram_token = std::env::var("TELEGRAM_TOKEN").unwrap();

        let core = tokio_core::reactor::Core::new().unwrap();
        let api = telegram_bot::Api::configure(telegram_token)
                                    .build(core.handle())
                                    .unwrap();

        ConfiguredApi {
            api,
            core,

            channel_id: -1001084499328,
            name: name.to_string(),
            parse_mode
        }
    }

    fn emit<T: Into<String>>(&mut self, msg: T) {
        let future = self.api.send(
            telegram_bot::types::requests::SendMessage::new(
                telegram_bot::types::ChatId::new(self.channel_id),
                format!("⥂ {} ⟹ {}", self.name, msg.into())
            )
            .parse_mode(self.parse_mode)
            .disable_preview()
        );

        self.core.run(future);
    }
}

fn main() {
    let mut configured_api = ConfiguredApi::new(
        &"<b>Reddit</b>",
        telegram_bot::types::ParseMode::Html
    );

    let kafka_topic_name = "reddit-comments";

    let kafka_hosts = vec![
        std::env::var("KAFKA_HOST").unwrap(),
        // "127.0.0.1:9092".to_string(),
    ];

    let kafka_consumer_group = "gjallarhorn-sat-1".to_string();

    let kafka_fetch_max_bytes_per_partition = 0x1ffffei32;

    let mut consumer =
            Consumer::from_hosts(kafka_hosts)
            .with_topic(kafka_topic_name.to_string())
            .with_fallback_offset(FetchOffset::Earliest)
            // .with_fallback_offset(FetchOffset::Latest)
            .with_group(kafka_consumer_group)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .with_fetch_max_bytes_per_partition(kafka_fetch_max_bytes_per_partition)
            .create()
            .unwrap();

    let mut i = 0;

    let matchers = [
        regex::Regex::new(r"(?i)psychonaut\s?wiki").unwrap(),
        regex::Regex::new(r"(?i)psychonaut.?wiki").unwrap(),
        regex::Regex::new(r"(?i)disregard\s?everything\s?i\s?say").unwrap()
    ];

    let reddit_rgx = regex::Regex::new(r"www.reddit.com").unwrap();

    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                i = i + 1;

                let rdata = &m.value.to_vec();
                let envelope_raw = String::from_utf8(rdata.to_vec()).unwrap();

                let mut envx: RedditPost = serde_json::from_str(&envelope_raw).unwrap();

                match envx.data.body {
                    Some(ref body) => {
                        if matchers.iter().any(|ref exp| exp.is_match(&body)) {
                            let author = envx.data.author.unwrap_or("unknown".to_string());
                            let link_author = envx.data.link_author.unwrap_or("unknown".to_string());

                            let target_url = if envx.data.permalink.is_some() {
                                format!(
                                    "https://www.reddit.com{}",
                                    
                                    envx.data.permalink.unwrap()
                                )
                            } else if !reddit_rgx.is_match(&envx.data.link_url.clone().unwrap_or("unknown".to_string())) {
                                let link_id = envx.data.link_id.unwrap_or("unknown".to_string());
                                let mut r_linkid = link_id.chars();
                                r_linkid.by_ref().nth(2);
                                let link_id = r_linkid.as_str();

                                format!(
                                    "https://www.reddit.com/r/{}/comments/{}/_/{}/",
                                    
                                    envx.data.subreddit.clone().unwrap_or("unknown".to_string()),
                                    link_id,
                                    envx.data.id.unwrap_or("unknown".to_string())
                                )
                            } else {
                                format!(
                                    "{}{}",
                                    
                                    envx.data.link_url.clone().unwrap_or("unknown".to_string()),
                                    envx.data.link_id.unwrap_or("unknown".to_string())
                                )
                            };

                            configured_api.emit(format!(
r#"<a href="{}">{}</a> commented in <a href="{}">/r/{}</a> on a post by <a href="{}">{}</a> ("{}"): {}

———
{}
———"#,
                                format!("https://www.reddit.com/user/{}", &author),
                                author,

                                format!("https://www.reddit.com/r/{}", envx.data.subreddit.clone().unwrap_or("unknown".to_string())),
                                envx.data.subreddit.clone().unwrap_or("unknown".to_string()),

                                format!("https://www.reddit.com/user/{}", &link_author),
                                link_author,

                                envx.data.link_title.unwrap_or("< .. didn't get a title from the reddit API .. >".to_string()),

                                target_url,

                                body
                            ));
                        }
                    },
                    _ => {}
                };

                // print!("{}\r\x1b[0J\x1b[1G\x1b[1;1H", i);
            }
        }
    }
}
