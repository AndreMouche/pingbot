use std::{fs::read_to_string, io::Error};
use std::collections::HashMap;
use serde::{Deserialize,Serialize};
use toml;
#[derive(Deserialize)]
#[derive(Serialize)]
pub struct Sig{
    #[serde(rename = "name")]
    pub name: String,
    #[serde(default)]
    #[serde(rename = "labels")]
    pub labels: Vec<String>,
    #[serde(rename = "slack-channel")]
    pub slack_channel: String,
    #[serde(default)]
    #[serde(rename = "slack-channel-in-tikv")]
    pub slack_workspace_in_tikv:bool,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "tidb-slack-token")]
    pub tidb_slack_token: String,
    #[serde(rename = "tikv-slack-token")]
    pub tikv_slack_token: String,
    #[serde(rename = "slack-channel")]
    pub slack_channel: String,
    #[serde(default)]
    #[serde(rename = "sigs")]
    pub sigs:Vec<Sig>,
    #[serde(rename = "github-token")]
    pub github_token: String,
    #[serde(default)]
    #[serde(rename = "repos")]
    pub repos: Vec<String>,
    #[serde(default)]
    #[serde(rename = "filter-labels")]
    pub filter_labels: Vec<String>,

    #[serde(rename = "discourse-base-url")]
    pub discourse_base_url: String,
    #[serde(default)]
    #[serde(rename = "discourse-categories")]
    pub discourse_categories: Vec<String>,
    #[serde(default)]
    #[serde(rename = "discourse-members")]
    pub discourse_members: Vec<String>,
}

impl Config {
    pub fn new(filename: String) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let config: Config = toml::from_str(&contents[..]).unwrap();
        Ok(config)
    }

    pub fn  get_labels_sig(&self)->HashMap<String,usize>{
        let mut labels_sig = HashMap::new();
        for (id,sig) in self.sigs.iter().enumerate(){
            for label in &sig.labels{
                labels_sig.insert(label.clone(),id);
            }
        }
        labels_sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn new_config() -> Result<Config, Error> {
        Config::new("config.example.toml".to_owned())
    }
 
    #[test]
    fn read_config_label(){
        let config =  Config::new("test.toml".to_owned()).unwrap();
        println!("config:{}",config.sigs.len());
        let mut labels_sig = HashMap::new();
        for (id,sig) in config.sigs.iter().enumerate(){
            println!("sig.label:{:?}",sig.labels);
             for label in &sig.labels{
                labels_sig.insert(label,id);
            }
            println!("in tidb:{}",sig.slack_workspace_in_tikv);
        }
        config.sigs.iter().enumerate().map(|(id,sig)|{
            println!("sig.label:{:?}",sig.labels);
            for label in &sig.labels{
                labels_sig.insert(label,id);
            }
        });
        println!("sig_labels:{:?}",labels_sig);
    }

    #[test]
    fn read_config() {
        let config = new_config().unwrap();
        // slack
        assert_eq!(config.tidb_slack_token, "slack-token");
        assert_eq!(config.slack_channel, "slack-channel");
        // github
        assert_eq!(config.github_token, "github-token");
        assert_eq!(config.repos, vec!("you06/pingbot"));
        assert_eq!(
            config.filter_labels,
            vec!("filter-label-1", "filter-label-2")
        );
        // discourse
        assert_eq!(config.discourse_base_url, "https://asktug.com");
        assert_eq!(
            config.discourse_categories,
            vec!("TiDB 用户问答", "TiDB 开发者")
        );
        assert_eq!(config.discourse_members, vec!("you06"));
         
        assert_eq!(config.sigs.len(),2);
        assert_eq!(config.sigs[0].name,"transaction");
        assert!(config.sigs[0].contains(&"label1".to_string()));
    }
}
/*
[sigs]
name="transaction"
filter-labels=["a","b"]
slack-channel="xxx"
*/
