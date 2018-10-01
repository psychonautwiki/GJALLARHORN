extern crate kafka;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

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

use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

fn main() {
    let kafka_topic_name = "redis-test-1";

    let kafka_hosts = vec![
        "127.0.0.1:9092".to_string(),
        // "kafka-stg-02.auctionata.web:9092".to_string(),
        // "kafka-stg-03.auctionata.web:9092".to_string()
    ];

    let kafka_consumer_group = "lotmonster-fooshipppt".to_string();

    let kafka_fetch_max_bytes_per_partition = 0xFFFFFi32;

    let mut consumer =
            Consumer::from_hosts(kafka_hosts)
            .with_topic(kafka_topic_name.to_string())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group(kafka_consumer_group)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .with_fetch_max_bytes_per_partition(kafka_fetch_max_bytes_per_partition)
            .create()
            .unwrap();

    let mut i = 0;

    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                i = i + 1;

                let rdata = &m.value.to_vec();
                let envelope_raw = String::from_utf8(rdata.to_vec()).unwrap();

                let envx: RedditPost = serde_json::from_str(&envelope_raw).unwrap();

                match envx.data.body {
                    Some(body) => println!("{:?}", body),
                    _ => {}
                };

                print!("{}\r\x1b[0J\x1b[1G\x1b[1;1H", i);
            }
        }
    }
}
