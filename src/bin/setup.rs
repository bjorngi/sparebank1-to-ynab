use sparebank1_to_ynab::sparebanken1;
use sparebank1_to_ynab::ynab::{Account, Budget, YnabClient};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::process::Command;
use std::{env, io};
use std::{
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
};

extern crate termion;
use rand::Rng;
use termion::color::{Fg, Red, Reset};

#[derive(Debug)]
pub struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

async fn get_access_token(
    code: &String,
    state: &String,
    client_id: &String,
    client_secret: &String,
) -> Result<AuthResponse, Box<dyn Error>> {
    let redirect_uri = "http://localhost:9050";
    let url = format!("https://api-auth.sparebank1.no/oauth/token?client_id={client_id}&client_secret={client_secret}&redirect_uri={redirect_uri}&grant_type=authorization_code&code={code}&state={state}");

    let response = reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_token = response["access_token"].as_str().unwrap().to_string();
    let refresh_token = response["refresh_token"].as_str().unwrap().to_string();

    Ok(AuthResponse {
        access_token,
        refresh_token,
    })
}

async fn get_sparebank1_auth_response(
    client_id: &String,
    client_secret: &String,
) -> Result<AuthResponse, Box<dyn Error>> {
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
    let auth_response = get_access_token(&code, &state, client_id, client_secret).await?;

    Ok(auth_response)
}

fn print_ynab_accounts(accounts: &[Account]) {
    println!("YNAB accounts:");
    for (index, account) in accounts.iter().enumerate() {
        println!("{}: {}", index + 1, account.name);
    }
}

fn select_budget(ynab_budgets: &[Budget]) -> &Budget {
    if ynab_budgets.len() == 1 {
        return ynab_budgets.first().expect("Nope");
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

        println!("YNAB Budgets: {:?}", ynab_budgets);
        println!("Choice: {}", choice);
        return ynab_budgets.get(choice - 1).expect("Do it");
    }
}

#[derive(Debug)]
pub struct Arguments {
    sparebank1_client_id: String,
    sparebank1_client_secret: String,
    sparebank1_fin_inst: String,
    ynab_access_token: String,
}

impl Arguments {
    // Function to create Arguments from command-line arguments
    fn from_args(args: Vec<String>) -> Result<Self, String> {
        // Check if the correct number of arguments is provided
        if args.len() != 4 {
            return Err(
                "Usage: program_name <sparebank1_client_id> <sparebank1_client_secret> <sparebank1_fint_inst> <ynab_access_token>"
                    .to_string(),
            );
        }

        // Assign command-line arguments to struct fields
        let sparebank1_client_id = args[0].clone();
        let sparebank1_client_secret = args[1].clone();
        let sparebank1_fin_inst = args[2].clone();
        let ynab_access_token = args[3].clone();

        // Create and return Arguments struct
        Ok(Self {
            sparebank1_client_id,
            sparebank1_client_secret,
            sparebank1_fin_inst,
            ynab_access_token,
        })
    }
}

fn write_config_file(
    sparebank1_client_id: &String,
    sparebank1_client_secret: &String,
    sparebank1_fin_inst: &String,
    ynab_access_token: &String,
    ynab_budget_id: &String,
    refresh_token: &String,
) -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let mut file = File::create("budget.env")?;

    // Default refresh token path
    let refresh_token_path = "refresh_token.txt";

    // Write environment variables to the file
    writeln!(file, "SPAREBANK1_CLIENT_ID={sparebank1_client_id}")?;
    writeln!(file, "SPAREBANK1_CLIENT_SECRET={sparebank1_client_secret}")?;
    writeln!(file, "SPAREBANK1_FIN_INST={sparebank1_fin_inst}")?;
    writeln!(file, "YNAB_BUDGET_ID={ynab_budget_id}")?;
    writeln!(file, "YNAB_ACCESS_TOKEN={ynab_access_token}")?;
    writeln!(file, "INITIAL_REFRESH_TOKEN={refresh_token}")?;
    writeln!(file, "ACCOUNT_CONFIG_PATH={}/accounts.json", cwd.display())?;
    writeln!(file, "REFRESH_TOKEN_FILE_PATH={}", refresh_token_path)?;

    println!("Config file created: {}/budget.env", cwd.display());

    // Save initial refresh token to the refresh token file
    std::fs::write(refresh_token_path, refresh_token)?;
    println!("Initial refresh token saved to: {}", refresh_token_path);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = rand::rng().random_range(100_000..1_000_000);
    let redirect_uri = "http://localhost:9050";
    let args: Vec<String> = env::args().skip(1).collect();
    let config = Arguments::from_args(args)?;

    let url = format!(
        "https://api-auth.sparebank1.no/oauth/authorize?client_id={}&state={}&redirect_uri={}&finInst={}&response_type=code",
        config.sparebank1_client_id,
        state,
        redirect_uri,
        config.sparebank1_fin_inst
    );

    // Open browser to start the OAuth flow
    let _ = open::that(url);
    let auth_response = get_sparebank1_auth_response(
        &config.sparebank1_client_id,
        &config.sparebank1_client_secret,
    )
    .await?;

    let sparebank1_accounts = sparebanken1::get_accounts(&auth_response.access_token).await?;

    // Create YnabClient with empty account config (we'll populate it later)
    let ynab_client = YnabClient::new(
        HashMap::new(),
        config.ynab_access_token.clone(),
        "".to_string(), // Initially an empty budget ID
    );

    // Get budgets using the client
    let ynab_budgets = ynab_client.get_budgets().await?;
    let selected_budget = select_budget(&ynab_budgets);

    // Create a client with the selected budget
    let ynab_client_with_budget = YnabClient::new(
        HashMap::new(),
        config.ynab_access_token.clone(),
        selected_budget.id.clone(),
    );

    // Get accounts using the client with the selected budget
    let ynab_accounts = ynab_client_with_budget.get_accounts().await?;

    if !std::path::Path::new("accounts.json").exists() {
        let account_config: HashMap<String, String> = sparebank1_accounts
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
                    let ynab_account_choice =
                        ynab_accounts.get(choice - 1).expect("Could not get choice");
                    config.insert(sb_acc.key.clone(), ynab_account_choice.id.clone());
                    Some(config.clone())
                } else {
                    Some(config.clone())
                }
            })
            .last()
            .unwrap_or_else(HashMap::new);

        println!("Config output: {:#?}", account_config);
        // Open or create the output file
        let mut file = File::create("accounts.json")?;
        // Serialize the vector to JSON and write it to the file
        let json_string = serde_json::to_string_pretty(&account_config)?;
        file.write_all(json_string.as_bytes())?;
    }

    write_config_file(
        &config.sparebank1_client_id,
        &config.sparebank1_client_secret,
        &config.sparebank1_fin_inst,
        &config.ynab_access_token,
        &selected_budget.id,
        &auth_response.refresh_token,
    )?;

    Ok(())
}
