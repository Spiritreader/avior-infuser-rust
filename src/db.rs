use crate::cfg::Config;
use mongodb::{
    bson::{self, doc, Bson},
    sync::Client as MongoClient
};
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct Client {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub name: String,
    pub availability_start: String,
    pub availability_end: String,
    pub maximum_jobs: i32,
    pub priority: i32,
    pub online: bool,
    pub ignore_online: bool
}


pub fn connect(cfg: &Config) -> Result<MongoClient, Box<dyn Error>> {
    //let conn_url = format!("mongodb://{}/", cfg.db_url);
    println!("connecting to {}", cfg.db_url);
    let client = MongoClient::with_uri_str(&cfg.db_url)?;
    Ok(client)
}

pub fn get_clients(mongo_client: MongoClient, db: &String) -> Result<Vec<Client>, Box<dyn Error>> {
    let db = mongo_client.database(&db);
    let collection = db.collection("clients");
    let cur = collection.find(doc!{}, None)?;
    let mut clients = Vec::new();
    for result in cur {
        match result {
            Ok(doc) => {
                let client: Client = bson::from_bson(Bson::Document(doc))?;
                clients.push(client);
            }
            Err(e) => eprintln!("Loop Error: {:?}", e)
        }
    }
    Ok(clients)
}
