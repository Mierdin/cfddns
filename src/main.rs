extern crate clap;
extern crate cloudflare;

use std::net::Ipv4Addr;
use std::str::FromStr;

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
        Ok(success) => println!("Success: {:#?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}:", status);
                for err in errors.errors {
                    println!("Error {}: {}", err.code, err.message);
                    for (k, v) in err.other {
                        println!("{}: {}", k, v);
                    }
                }
                for (k, v) in errors.other {
                    println!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(reqwest_err) => println!("Error: {}", reqwest_err),
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

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
    // TODO - consider adding a lookup function for these two
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
    .setting(AppSettings::ArgRequiredElseHelp);

    let matches = cli.get_matches();

    let email = matches.value_of("email");
    let key = matches.value_of("auth-key");
    let token = matches.value_of("auth-token");
    let zone_id = matches.value_of("zone-id").unwrap();
    let record_id = matches.value_of("record-id").unwrap();

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

    // TODO - handle error
    let addr = get_ip_addr().unwrap();

    let api_client = HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let response = api_client.request(&dns::UpdateDnsRecord {
        identifier: record_id,
        zone_identifier: zone_id,
        params: dns::UpdateDnsRecordParams {
            ttl: Some(1),
            proxied: Some(false),

            // TODO - configurable
            name: "remote",

            // TODO - add v6
            content: dns::DnsContent::A{
                content: addr,
            }
        },
    });

    print_response(response);

    Ok(())
}

fn get_ip_addr() -> Result<Ipv4Addr, Box<dyn std::error::Error>>{
    let addr = reqwest::blocking::get("https://api.ipify.org/")?
    .text()?;
    println!("IP is {}", addr);

    // TODO - detect std::net::AddrParseError
    let addr = Ipv4Addr::from_str(&addr)?;

    Ok(addr)
}
