pub extern crate chrono;
pub extern crate tokio_core;
pub extern crate futures;

use std;
use std::io::{Write, Read};
use egg_mode;

use self::tokio_core::reactor::Core;

pub struct Config {
    pub token: egg_mode::Token,
    pub user_id: u64,
    pub screen_name: String,
}

impl Config {
    pub fn load(core: &mut Core) -> Self {
        let consumer_key = "IQKbtAYlXLripLGPWd0HUA";
        let consumer_secret = "GgDYlkSvaPxGxC4X8liwpUoqKwwr3lCADbz8A7ADU";
        let handle = core.handle();

        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let mut config = String::new();
        let user_id: u64;
        let username: String;
        let token: egg_mode::Token;

        if let Ok(mut f) = std::fs::File::open("twitter_settings") {
            f.read_to_string(&mut config).unwrap();

            let mut iter = config.split('\n');

            username = iter.next().unwrap().to_string();
            user_id = u64::from_str_radix(&iter.next().unwrap(), 10).unwrap();
            let access_token = egg_mode::KeyPair::new(iter.next().unwrap().to_string(),
                                                      iter.next().unwrap().to_string());
            token = egg_mode::Token::Access {
                consumer: con_token,
                access: access_token,
            };

            if let Err(err) = core.run(egg_mode::verify_tokens(&token, &handle)) {
                invalid_token(err);
            } else {
                println!("--------------------------------------------------------------------------------");
            }
        } else {
            let request_token = core.run(egg_mode::request_token(&con_token, "oob", &handle)).unwrap();
            let pin = get_pin(&request_token);
            let tok_result = core.run(egg_mode::access_token(con_token, &request_token, pin, &handle)).unwrap();

            token = tok_result.0;
            user_id = tok_result.1;
            username = tok_result.2;

            update_config(&token, &mut config, user_id, &username);
            create_settings(config);

            println!("Welcome, {}, let's get this show on the road!", username);
        }

        if std::fs::metadata("twitter_settings").is_ok() {
            return Config {
                token: token,
                user_id: user_id,
                screen_name: username,
            };
        } else {
            return Self::load(core);
        }
    }
}


fn invalid_token(err: egg_mode::error::Error) {
    println!("We've hit an error using your old tokens: {:?}", err);
    println!("We'll have to reauthenticate before continuing.");
    std::fs::remove_file("twitter_settings").unwrap();
}


fn get_pin(request_token: &egg_mode::KeyPair) -> String {
    println!("Go to the following URL, sign in, and give me the PIN that comes back:");
    println!("{}", egg_mode::authorize_url(request_token));

    let mut pin = String::new();
    std::io::stdin().read_line(&mut pin).unwrap();
    println!("");
    return pin;
}


fn update_config(token: &egg_mode::Token, config: &mut String, user_id: u64, username: &String) {
    match token {
        egg_mode::Token::Access { access: ref access_token, .. } => {
            config.push_str(username);
            config.push('\n');
            config.push_str(&format!("{}", user_id));
            config.push('\n');
            config.push_str(&access_token.key);
            config.push('\n');
            config.push_str(&access_token.secret);
        },
        _ => unreachable!(),
    }
}


fn create_settings(config: String) {
    let mut f = std::fs::File::create("twitter_settings").unwrap();
    f.write_all(config.as_bytes()).unwrap();
}
