mod config;
mod providers;

use clap::Clap;
use config::{Config,Sig};
use providers::discourse::Discourse;
use providers::github::GitHub;
use providers::slack::Slack;

#[derive(Clap)]
#[clap(version = "1.0", author = "you06")]
struct Opts {
    #[clap(short = "c", long = "config", default_value = "config.toml")]
    config: String,
    #[clap(short = "p", long = "ping")]
    ping: Option<String>,
}

pub struct SIGInfo<'a> {
    number:usize,
    report:String,
    sig:&'a Sig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let conf = Config::new(opts.config).unwrap();
    let (tidb_client,tikv_client)={
        let tidb_client=Slack::new(conf.tidb_slack_token.clone());
        let tikv_client=Slack::new(conf.tikv_slack_token.clone());
        (tidb_client,tikv_client)
    };
    if let Some(ping) = opts.ping {
        let _ = tidb_client
            .send_message(conf.slack_channel.clone(), ping.clone())
            .await?;
        // for sig in conf.sigs{
        //     println!("Current channel: {}", sig.slack_channel);
        //      if sig.slack_workspace_in_tikv {
        //          tikv_client.send_message(sig.slack_channel.clone(), ping.clone()).await?;
        //      }else {
        //         tidb_client.send_message(sig.slack_channel.clone(), ping.clone()).await?;
        //      }
        // }
        return Ok(());
    }

    let mut has_topic = false;

    let github_client = GitHub::new(conf.github_token.to_owned(), conf.filter_labels.clone());
    let user = github_client.get_user_result().await;
    println!("Current user: {}", user.unwrap());

    let issues = github_client.get_opened_issues(conf.repos.clone()).await?;
    let labels_sig=conf.get_labels_sig();
    let mut sig_list = vec![];
    for sig in &conf.sigs{
        sig_list.push(
            SIGInfo{
                report:"".to_owned(),
                number:0,
                sig:sig
            }
        );
    }
    let mut oncall = "".to_owned();
    let mut un_dispatch_issues = issues.len();
    for issue in issues {
        println!("issues.repo {}",issue.repo);
        let mut find=false;
        // check whether this repo has a full owner
        if let Some(id)=labels_sig.get(&issue.repo) {
             sig_list[*id].report.push_str(&format!("{}\n", issue)[..]);
             sig_list[*id].number += 1;
             un_dispatch_issues -= 1;
             continue;
        }
        for label in &issue.labels {
            let lower_label = format!("{}:{}",issue.repo,label.name.to_lowercase());
            println!("lower_label:{}",lower_label);
            if let Some(id)= labels_sig.get(&lower_label) {
                    sig_list[*id].report.push_str(&format!("{}\n", issue)[..]);
                    sig_list[*id].number += 1;
                    find=true;
                    break;
                }
        }
        if !find {
            oncall.push_str(&format!("{}\n", issue)[..]);
            un_dispatch_issues -= 1;
        }
    }

    for sig in sig_list{
        if sig.number == 0 {
            continue;
        }
        let mut report=format!("{} no-reply issues \n", sig.number);
        report.push_str(&sig.report);
        println!("---------{}-----",sig.sig.name);
        println!("{}",report);
        if sig.sig.slack_workspace_in_tikv {
           tikv_client.send_message(sig.sig.slack_channel.clone(),report).await?;
        } else {
           tidb_client.send_message(sig.sig.slack_channel.clone(),report).await?;
        }   
    }
    println!("---total-oncall---");
    println!("{}",oncall);
    let mut report = format!("{} no-reply issues \n", un_dispatch_issues);
    report.push_str(&oncall);
    // let discourse_client = Discourse::new(
    //     conf.discourse_base_url.to_owned(),
    //     conf.discourse_members.clone(),
    // );
    // let topics = discourse_client
    //     .find_no_reply_topics_by_categories(conf.discourse_categories.clone())
    //     .await?;

    // if topics.len() != 0 {
    //     has_topic = true;
    //     report.push_str(&format!("\n\n{} no-reply topics in TUG\n", topics.len())[..]);
    //     for topic in topics {
    //       report.push_str(&format!("{}\n", topic)[..]);
    //     }
    // }

    if conf.tidb_slack_token != "" && conf.slack_channel != "" {
        if un_dispatch_issues>0 || has_topic {
            let slack_client = Slack::new(conf.tidb_slack_token.clone());
            let _ = slack_client
                .send_message(conf.slack_channel.clone(), report)
                .await?;
        }
    } else {
        println!("{}", report);
    }
    Ok(())
}
