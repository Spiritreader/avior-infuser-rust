use avior_infuser_lib::*;
use log::Log;
use log::Logger;
use std::{env, error::Error};

const CFG_PATH: &str = "config.json";
const IDENTITY: &str = "avior infuser rust, version 0.1 - maneki-neko";

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", IDENTITY);
    let args: Vec<String> = env::args().collect();
    println!("calling args: {:?}", &args[1..]);
    if args.len() < 4 {
        return Err("program needs exactly 3 arguments in this order: path, name, sub".into());
    }

    let mut logger: Logger = Log::new(IDENTITY);

    let config = cfg::read(CFG_PATH)?;
    let mongo_client = db::connect(&config)?;
    let _ = db::get_jobs(&mongo_client, &config.db_name)?;
    let client_vec: Vec<db::Client> = db::get_clients(&mongo_client, &config.db_name)?;

    //client_vec.sort_by(|a, b| b.priority.cmp(&a.priority));
    let grouped_clients = group_clients(&client_vec);
    let machine_jobcounts = db::get_machine_jobcount(&mongo_client, &config.db_name)?;

    let mut new_job = db::Job {
        id: None,
        path: args[1].to_string(),
        name: args[2].to_string(),
        subtitle: args[3].to_string(),
        assigned_client: db::AssignedClient::default(),
        custom_parameters: Vec::new(),
    };

    // if job exists we good
    if db::job_exists(&mongo_client, &config.db_name, &new_job.path)? {
        println!("we good, job already exists in database");
        return Ok(());
    }

    // try pushing job to eligible client
    let mut result =
        get_eligible_client(grouped_clients, machine_jobcounts).and_then(|(eligible_id, count, maximum)| {
            if let Some(found) = client_vec
                .iter()
                .find(|client| client.id.to_owned().unwrap_or_default().to_string() == eligible_id)
            {
                let iid = db::insert_job(&mongo_client, &config.db_name, found, &mut new_job)?;
                logger.add(&format!(
                    "found eligible client {} with {}/{} job(s) and priority {}",
                    found.name, count, maximum, found.priority
                ));
                return Ok(iid);
            }
            Err(InfuserError {
                message: "could not find eligible client candidate in client_vec".into(),
            }
            .into())
        });

    // try pushing job to default client
    if let Err(e) = result {
        println!("{:?}, pushing to default client {}", e, &config.default_client);
        result = client_vec
            .iter()
            .find(|client| client.name == config.default_client)
            .ok_or(
                InfuserError {
                    message: "could not find eligible default client in client_vec".into(),
                }
                .into(),
            )
            .and_then(|found| {
                let iid = db::insert_job(&mongo_client, &config.db_name, found, &mut new_job)?;
                Ok(iid)
            });
    }
    match result {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
        }
    }
    Ok(())
}

/*
fn push_to_eligible(
    grouped_clients: BTreeMap<i32, Vec<&db::Client>>,
    machine_jobcounts: HashMap<String, i32>,
    mongo_client: &mongodb::sync::Client,
    config: &cfg::Config,
    new_job: &mut db::Job,
    client_vec: &Vec<db::Client>,
) -> Result<(), Box<dyn Error>> {
    match get_eligible_client(grouped_clients, machine_jobcounts) {
        Ok(eligible_id) => {
            if db::job_exists(mongo_client, &config.db_name, &new_job.path)? {
                return Err("job already exists".into());
            }
            let found = client_vec
                .iter()
                .find(|client| client.id.to_owned().unwrap_or_default().to_string() == eligible_id);

            if let Some(client) = found {
                match db::insert_job(&mongo_client, &config.db_name, client, new_job) {
                    Ok(_) => {
                        println!("pushed job to {}", &client.name);
                        // exit if success
                        return Ok(());
                    }
                    Err(e) => {
                        println!("{:?}, couldn't insert job:", e);
                        return Err(Box::new(e));
                    }
                }
            } else {
                return Err("client did not exist".into());
            }
        }
        Err(e) => {
            println!("{:?}, pushing to default client instead", e);
            return Err(e);
        }
    }
}*/
