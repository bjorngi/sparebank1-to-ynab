mod config;
mod sparebanken1;
mod ynab;

use crate::config::Config;

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::process::Command;
use std::{
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
};

extern crate termion;
use termion::color::{Blue, Fg, Green, Red, Reset};

#[derive(Debug)]
pub struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

async fn get_access_token(
    code: &String,
    state: &String,
    config: &Config,
) -> Result<AuthResponse, Box<dyn Error>> {
    let client_id = &config.sparebank1_client_id;
    let client_secret = &config.sparebank1_client_secret;
    let redirect_uri = "http://localhost:9050";
    let url = format!("https://api-auth.sparebank1.no/oauth/token?client_id={client_id}&client_secret={client_secret}&redirect_uri={redirect_uri}&grant_type=authorization_code&code={code}&state={state}");

    let response = reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("Response: {:#?}", response);

    let access_token = response["access_token"].as_str().unwrap().to_string();
    let refresh_token = response["refresh_token"].as_str().unwrap().to_string();

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

async fn get_sparebank1_auth_response(config: &Config) -> Result<AuthResponse, Box<dyn Error>> {
    fn get_code_and_state_from_response(response: &str) -> (String, String) {
        let code = response.split("code=").collect::<Vec<&str>>()[1]
            .split('&')
            .collect::<Vec<&str>>()[0]
            .to_string();
        let state = response.split("state=").collect::<Vec<&str>>()[1]
            .split('&')
            .collect::<Vec<&str>>()[0]
            .to_string();

        (code, state)
    }

    fn handle_client(mut stream: TcpStream) -> (String, String) {
        let buf_reader = BufReader::new(&mut stream);
        let url: Option<String> = buf_reader.lines().map(|result| result.unwrap()).next();

        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();

        get_code_and_state_from_response(&url.expect("No URL found"))
    }

    let addr: SocketAddr = ([127, 0, 0, 1], 9050).into();
    let listener = TcpListener::bind(addr).expect("Failed to bind to address");
    let (code, state) = handle_client(listener.accept().unwrap().0);
    let auth_response = get_access_token(&code, &state, config).await?;

    Ok(auth_response)
}

fn print_ynab_accounts(accounts: &[ynab::Account]) {
    println!("YNAB accounts:");
    for (index, account) in accounts.iter().enumerate() {
        println!("{}: {}", index + 1, account.name);
    }
}

fn select_budget(ynab_budgets: &[ynab::Budget]) -> &ynab::Budget {
    if ynab_budgets.len() == 1 {
        return ynab_budgets.get(0).expect("Nope").clone();
    } else {
        println!("YNAB Budgets:");
        for (index, budget) in ynab_budgets.iter().enumerate() {
            println!("{}: {}", (index + 1), budget.name)
        }

        println!("Select budget to use: ");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let choice: usize = input.trim().parse().expect("Must be a number");
        return ynab_budgets.get(choice - 1).expect("Do it").clone();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = 7929309;
    let redirect_uri = "http://localhost:9050";
    let fin_inst = "fid-ostlandet";
    let config = Config::new()?;
    let client_id = &config.sparebank1_client_id;

    let url = format!("https://api-auth.sparebank1.no/oauth/authorize?client_id={client_id}&state={state}&redirect_uri={redirect_uri}&finInst={fin_inst}&response_type=code");

    // Open browser to start the OAuth flow
    let _ = open::that(url);
    let auth_response = get_sparebank1_auth_response(&config).await?;

    let sparebank1_accounts = sparebanken1::get_accounts(&auth_response.access_token).await?;
    let ynab_budgets = ynab::get_budgets(&config).await?;
    let selected_budget = select_budget(&ynab_budgets);

    let ynab_accounts = ynab::get_accounts(&config).await?;

    let config_output: HashMap<String, String> = sparebank1_accounts
        .iter()
        .enumerate()
        .scan(HashMap::new(), |config, (_, sb_acc)| {
            let _ = Command::new("clear").status();
            println!(
                "Account setup for budget:{}{}{}",
                Fg(Red),
                selected_budget.name,
                Fg(Reset)
            );
            print_ynab_accounts(&ynab_accounts);
            println!("{}{}{} -- link to", Fg(Red), sb_acc.name, Fg(Reset));
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let choice: usize = input.trim().parse().expect("Must be a number");

            if choice > 0 {
                let ynab_account_choice = ynab_accounts.get(choice)?;
                config.insert(sb_acc.key.clone(), ynab_account_choice.id.clone());
                Some(config.clone())
            } else {
                Some(config.clone())
            }
        })
        .last()
        .unwrap_or_else(HashMap::new);

    println!("Config output: {:#?}", config_output);

    Ok(())
}
