extern crate clap;
extern crate cloudflare;

use std::net::Ipv4Addr;
use std::str::FromStr;
use std::{thread, time};
use log::{info, debug, warn};

use clap::{App, AppSettings, Arg};
use cloudflare::endpoints::dns;
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    response::{ApiFailure, ApiResponse, ApiResult},
    Environment, HttpApiClient, HttpApiClientConfig,
};

fn print_response<T: ApiResult>(response: ApiResponse<T>) {
    match response {
        Ok(success) => debug!("Success: {:#?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                warn!("HTTP {}:", status);
                for err in errors.errors {
                    warn!("Error {}: {}", err.code, err.message);
                    for (k, v) in err.other {
                        warn!("{}: {}", k, v);
                    }
                }
                for (k, v) in errors.other {
                    warn!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(reqwest_err) => warn!("Error: {}", reqwest_err),
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = App::new("cfddns")
    .version("0.1")
    .author("Matt Oswalt")
    .about("Simple dynamic DNS utility for Cloudflare")
    .arg(Arg::with_name("email")
        .long("email")
        .help("Email address associated with your account")
        .takes_value(true)
        .requires("auth-key"))
    .arg(Arg::with_name("auth-key")
        .long("auth-key")
        .env("CFDDNS_AUTH_KEY")
        .help("API key generated on the \"My Account\" page")
        .takes_value(true)
        .requires("email"))
    .arg(Arg::with_name("auth-token")
        .long("auth-token")
        .env("CFDDNS_AUTH_TOKEN")
        .help("API token generated on the \"My Account\" page")
        .takes_value(true)
        .conflicts_with_all(&["email", "auth-key"]))
    .arg(Arg::with_name("zone-id")
        .long("zone-id")
        .env("CFDDNS_ZONE_ID")
        .help("Zone ID that contains record to update")
        .takes_value(true))
    .arg(Arg::with_name("record-id")
        .long("record-id")
        .env("CFDDNS_RECORD_ID")
        .help("ID of record to update")
        .takes_value(true))
    .arg(Arg::with_name("record-name")
        .long("record-name")
        .env("CFDDNS_RECORD_NAME")
        .help("Name of record to update")
        .takes_value(true))
    .arg(Arg::with_name("interval")
        .long("interval")
        .env("CFDDNS_INTERVAL")
        .help("Interval (in seconds) to wait in between updates (defaults to 1800)")
        .takes_value(true))
    .setting(AppSettings::ArgRequiredElseHelp);

    let matches = cli.get_matches();

    let email = matches.value_of("email");
    let key = matches.value_of("auth-key");
    let token = matches.value_of("auth-token");
    let zone_id = matches.value_of("zone-id").unwrap();
    let record_name = matches.value_of("record-name").unwrap();
    let record_id = matches.value_of("record-id").unwrap();
    let interval = match matches.value_of("interval") {
        None => 1800 as u64,
        Some(s) => {
            match s.parse::<u64>() {
                Ok(n) => n,
                Err(_) => 1800 as u64,
            }
        }
    };

    let credentials: Credentials = if let Some(key) = key {
        Credentials::UserAuthKey {
            email: email.unwrap().to_string(),
            key: key.to_string(),
        }
    } else if let Some(token) = token {
        Credentials::UserAuthToken {
            token: token.to_string(),
        }
    } else {
        panic!("Either API token or API key + email pair must be provided")
    };

    let api_client = HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    loop {
        update(
            &api_client,
            record_id,
            record_name,
            zone_id
        )?;

        info!("Update successful. Sleeping for {} seconds.", interval);
        thread::sleep(time::Duration::from_secs(interval));
    }
}

fn get_ip_addr() -> Result<Ipv4Addr, Box<dyn std::error::Error>>{
    let addr = reqwest::blocking::get("https://api.ipify.org/")?
    .text()?;
    info!("Detected IPv4 address: {}", addr);

    // TODO - detect std::net::AddrParseError
    let addr = Ipv4Addr::from_str(&addr)?;

    Ok(addr)
}

fn update<ApiClientType: ApiClient>(
    api_client: &ApiClientType,
    record_id: &str,
    record_name: &str,
    zone_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let addr = get_ip_addr()?;

    // TODO - check to see if an update is needed first?

    let response = api_client.request(&dns::UpdateDnsRecord {
        identifier: record_id,
        zone_identifier: zone_id,
        params: dns::UpdateDnsRecordParams {
            ttl: Some(1),
            proxied: Some(false),
            name: record_name,
            // TODO - add v6
            content: dns::DnsContent::A{
                content: addr,
            }
        },
    });

    print_response(response);

    Ok(())
}