mod cfg;
mod db;
use std::collections::BTreeMap;
use std::{collections::HashMap, fmt};
use std::{env, error::Error};

const CFG_PATH: &str = "config.json";

pub struct NoEligibleClientError {
    pub message: String,
}

impl fmt::Debug for NoEligibleClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for NoEligibleClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for NoEligibleClientError {}

fn main() -> Result<(), Box<dyn Error>> {
    println!("avior infuser rust, version 0.1 - maneki-neko");
    let args: Vec<String> = env::args().collect();
    println!("calling args: {:?}", &args);
    if args.len() < 4 {
        return Err("program needs exactly 3 arguments in this order: path, name, sub".into());
    }
    let config = cfg::read(CFG_PATH)?;
    let mongo_client = db::connect(&config)?;
    let _ = db::get_jobs(&mongo_client, &config.db_name)?;
    let client_vec: Vec<db::Client> = db::get_clients(&mongo_client, &config.db_name)?;

    //client_vec.sort_by(|a, b| b.priority.cmp(&a.priority));
    let grouped_clients = group_clients(&client_vec)?;
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
    let result = get_eligible_client(grouped_clients, machine_jobcounts).and_then(|eligible_id| {
        if let Some(found) = client_vec
            .iter()
            .find(|client| client.id.to_owned().unwrap_or_default().to_string() == eligible_id)
        {
            println!("pushing job to {}", &found.name);
            return Ok(db::insert_job(&mongo_client, &config.db_name, found, &mut new_job)?);
        }
        Err("could not find eligible client candidate in client_vec".into())
    });

    // try pushing job to default client
    if let Err(e) = result {
        println!("{:?}, pushing to default client {}", e, &config.default_client);
        let default_client = client_vec.iter().find(|client| client.name == config.default_client);
        match default_client {
            Some(found) => {
                db::insert_job(&mongo_client, &config.db_name, found, &mut new_job)?;
            }
            None => return Err("could not find default candidate in client_vec".into()),
        }
    }
    Ok(())
}

fn get_eligible_client(
    grouped_clients: BTreeMap<i32, Vec<&db::Client>>,
    machine_jobcounts: HashMap<String, i32>,
) -> Result<String, Box<dyn Error>> {
    for (_, clients) in grouped_clients {
        let mut lowest = i32::MAX;
        let mut eligible: Option<db::Client> = None;
        // loop over every client within a priority group
        // rules: get the client...
        // - with the lowest jobcount
        // - that is online or has the ignore_online flag enabled
        // - that hasn't reached its maximum job count
        for client in clients {
            let key = client
                .id
                .to_owned()
                .ok_or(NoEligibleClientError {
                    message: "a client in the database has no id, could not determine eligible clients".to_string(),
                })?
                .to_string();
            if !client.online && !client.ignore_online {
                continue;
            }
            if let Some(count) = machine_jobcounts.get(&key) {
                if *count < lowest && *count < client.maximum_jobs {
                    eligible = Some(client.to_owned());
                    lowest = *count;
                }
            } else {
                eligible = Some(client.to_owned());
                lowest = 0;
            }
        }
        // if a client was found within the priority group,
        // return it, otherwise move on to the next one
        match eligible {
            Some(client) => {
                println!("found eligible client {} with {} jobs", client.name, lowest);
                return Ok(client.id.to_owned().unwrap().to_string());
            }
            None => (),
        }
    }
    // if no client has been found, return an error
    Err(Box::new(NoEligibleClientError {
        message: "no eligible client found".to_string(),
    }))
}

fn group_clients(client_vec: &Vec<db::Client>) -> Result<BTreeMap<i32, Vec<&db::Client>>, Box<dyn Error>> {
    let mut dict = BTreeMap::new();
    for client in client_vec {
        let prio = client.priority;
        dict.entry(prio).or_insert(Vec::new()).push(client);
        /*
        match dict.entry(prio) {
            Entry::Vacant(e) => e.insert(vec![client]);
            Entry::Occupied(mut e) => {}
        }
         */
    }
    Ok(dict)
}

#[allow(dead_code)]
fn test_insert(
    client: &mongodb::sync::Client,
    insert_client: &db::Client,
    config: &cfg::Config,
) -> Result<(), Box<dyn Error>> {
    let iid = db::insert_job(client, &config.db_name, insert_client, &mut db::Job {
        id: None,
        path: "\\\\vdr-u\\SDuRec\\Recording\\exists\\Geheimnisvolle Wildblumen_2021-04-10-14-58-01-arte HD (AC3,deu).ts".to_string(),
        name: "Geheimnisvolle Wildblumen".to_string(),
        subtitle: "Blütenpracht im Wald".to_string(),
        assigned_client: db::AssignedClient::default(),
        custom_parameters: Vec::new()
    })?;
    println!("{}", iid);
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
