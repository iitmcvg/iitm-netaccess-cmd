// netacess: A command line tool to approve/revoke internet access at IIT Madras
// Copyright (C) 2017 Jithin Jith
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate rpassword;
extern crate reqwest;
extern crate rand;
extern crate pnet;

#[macro_use]
extern crate clap;

use std::io::{self, Write};
use std::fmt;
use std::collections::HashMap;
use rand::Rng;

/*
fn get_credentials() -> (String, String)
{
    let mut username = String::new();
    print!("Username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).expect("Failed to read line");

    let password = rpassword::prompt_password_stdout("Password: ").unwrap();

    (String::from(username.trim()), password)
}
*/

fn get_username() -> String
{
    let mut username = String::new();
    print!("Username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).expect("Failed to read line");
    String::from(username.trim())
}

fn get_password() -> String
{
    let password = rpassword::prompt_password_stdout("Password: ").unwrap();
    password
}

fn get_approval_duration() -> (char)
{
    let mut duration_str = String::new();
    let mut duration: char;

    loop
    {
        print!("Session duration (1: one hour (default), 2: one day): ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut duration_str).expect("Failed to read line");

        if duration_str.trim().is_empty()
        {
            duration = '1';
            break;
        }
        else
        {
            duration = duration_str.chars().nth(0).unwrap();

            match duration
            {
                '1' =>
                {
                    println!("You have requested approval for one hour");
                    break;
                },
                '2' =>
                {
                    println!("You have requested approval for one day");
                    break;
                },
                _ =>
                {
                    println!("Enter a valid option");
                    duration_str.clear();
                },
            }

            /*
            if duration == '1'
            {
                println!("You have requested approval for one hour");
                break;
            }
            else if duration == '2'
            {
                println!("You have requested approval for one day");
                break;
            }
            else
            {
                println!("Enter a valid option");
                duration_str.clear();
            }
            */
        }
    }

    duration
}

fn create_cookie() -> reqwest::header::Cookie
{
    let mut cookie = reqwest::header::Cookie::new();
    let mut sess_id = String::from("iitm-netaccess-cmd");
    let mut rand_num;

    for _j in 1..6
    {
        rand_num = rand::thread_rng().gen_range(1, 101);
        sess_id.push_str(&(rand_num.to_string()));
    }

    cookie.append("PHPSESSID", sess_id);
    cookie
}

fn create_headers() -> reqwest::header::Headers
{
    let user_agent = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:49.0) Gecko/20100101 Firefox/49.0";

    let mut my_header = reqwest::header::Headers::new();
    my_header.set(reqwest::header::UserAgent::new(user_agent));
    my_header.set(create_cookie());

    my_header
}

fn serialize_credentials(username: String, password: String) -> HashMap<String,String>
{
    let mut data = HashMap::new();
    data.insert("userLogin".to_string(), username);
    data.insert("userPassword".to_string(), password);

    data
}

fn do_login(my_url: reqwest::Url, username: String, password: String, my_headers: reqwest::header::Headers) -> bool
{
    let my_data = serialize_credentials(username, password);

    let client = reqwest::Client::new();
    let mut res = client.post(my_url)
        .headers(my_headers)
        .form(&my_data)
        .send()
        .expect("Failed to send login request");

    let mut my_status = true;
    if res.status() != reqwest::StatusCode::Ok
    {
        my_status = false;
        println!("Error logging in. Return status {}.", res.status());
    }
    else
    {
        if !res.text().unwrap().contains("/account/approve")
        {
            my_status = false;
            println!("Failed to login. Wrong credentials supplied.");
        }
        else
        {
            println!("Login successful");
        }
    }

    my_status

}

fn serialize_duration(duration: char) -> HashMap<String,char>
{
    let mut data = HashMap::new();
    data.insert("duration".to_string(), duration);
    data.insert("approveBtn".to_string(), ' ');

    data
}

fn do_approve(my_url: reqwest::Url, duration: char, my_headers: reqwest::header::Headers) -> bool
{
    let my_data = serialize_duration(duration);

    let client = reqwest::Client::new();
    let res = client.post(my_url)
        .headers(my_headers)
        .form(&my_data)
        .send()
        .expect("Failed to send approval request");

    let mut my_status = true;
    if res.status() != reqwest::StatusCode::Ok
    {
        my_status = false;
        println!("Error approving. Returned status {}.", res.status());
    }
    else
    {
        println!("Succesfully approved");
    }

    my_status
}

fn get_ip_address() -> String
{
    let ifaces = pnet::datalink::interfaces();

    println!("Select network interface:");
    for j in 0..ifaces.len()
    {
        print!("({}) {} ", j, (ifaces[j].name).clone());
    }
    print!(": ");
    io::stdout().flush().unwrap();

    let mut selection_str = String::new();
    let mut selection: Result<u8,std::num::ParseIntError>;

    loop
    {
        io::stdin().read_line(&mut selection_str).expect("Failed to read line");
        selection = selection_str.trim().parse();

        if selection.is_err() || (selection.clone()).unwrap() >= (ifaces.len() as u8)
        {
            selection_str.clear();
            print!("Enter a valid option: ");
            io::stdout().flush().unwrap();
        }
        else
        {
            break;
        }
    }

    let address = &ifaces[selection.unwrap() as usize].ips;

    let mut output = String::new();

    for ad in address
    {
        if (*ad).is_ipv4()
        {
            fmt::write(&mut output, format_args!("{}", (*ad).ip()))
            .expect("Error writing IP address to string!");
            break
        }
    }

    output
}

fn do_revoke(my_url: reqwest::Url, ip: Option<String>, my_headers: reqwest::header::Headers) -> bool
{
    //let my_ip = my_ip.unwrap_or(get_ip_address());

    let mut my_ip = ip;
    if my_ip.is_none()
    {
        my_ip = Some(get_ip_address());
    }
    let my_ip = my_ip.unwrap();

    let full_url = my_url.join(&my_ip).unwrap();

    //println!("{}", full_url);

    let client = reqwest::Client::new();
    let res = client.get(full_url)
        .headers(my_headers)
        .send()
        .expect("Failed to send revoke request");

    let mut my_status = true;
    if res.status() != reqwest::StatusCode::Ok
    {
        my_status = false;
        println!("Error revoking. Returned status {}.", res.status());
    }
    else
    {
        println!("Succesfully revoked {}", my_ip);
    }

    my_status
}

fn cmd_approve(login_url: reqwest::Url, approve_url: reqwest::Url,
               username: Option<String>, password: Option<String>, duration: Option<String>)
{
    let username = match username
    {
        Some(u) => u,
        _ => get_username(),
    };

    let password = match password
    {
        Some(p) => p,
        _ => get_password(),
    };

    let duration = match duration
    {
        Some(d) => d.chars().nth(0).unwrap(),
        _ => get_approval_duration(),
    };

    let my_headers = create_headers();
    if do_login(login_url, username, password, my_headers.clone())
    {
        do_approve(approve_url, duration, my_headers);
    }
}

fn cmd_revoke(login_url: reqwest::Url, revoke_url: reqwest::Url, ip: Option<String>,
              username: Option<String>, password: Option<String>)
{
    let username = match username
    {
        Some(u) => u,
        _ => get_username(),
    };

    let password = match password
    {
        Some(p) => p,
        _ => get_password(),
    };

    let my_headers = create_headers();
    if do_login(login_url, username, password, my_headers.clone())
    {
        do_revoke(revoke_url, ip, my_headers);
    }
}

fn main()
{
    let login_url = reqwest::Url::parse("https://netaccess.iitm.ac.in/account/login/").unwrap();
    let approve_url = reqwest::Url::parse("https://netaccess.iitm.ac.in/account/approve/").unwrap();
    let revoke_url = reqwest::Url::parse("https://netaccess.iitm.ac.in/account/revoke/").unwrap();

    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();

    let username = matches.value_of("username").map(String::from);
    let password = matches.value_of("password").map(String::from);
    let duration = matches.value_of("duration").map(String::from);

    match matches.subcommand()
    {
        ("approve", Some(_cmd)) =>
        {
            cmd_approve(login_url, approve_url, username, password, duration);
        },
        ("revoke", Some(cmd)) =>
        {
            let ip = cmd.value_of("ip").map(String::from);
            cmd_revoke(login_url, revoke_url, ip, username, password);
        },
        _ =>
        {
            cmd_approve(login_url, approve_url, username, password, duration);
        }
    }
}
