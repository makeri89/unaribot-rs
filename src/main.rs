use teloxide::{prelude::*, utils::command::BotCommands, types::ParseMode, utils::markdown};
use std::error::Error;
use serde::{Serialize, Deserialize};

static UNICAFE_BASE_URL: &str = "https://unicafe.fi/wp-json/swiss/v1/restaurants/?lang=fi";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MenuData {
    name: String,
    id: i32,
    areacode: i32,
    menus: Vec<MenuList>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Price {
    name: String,
    value: PriceOptions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PriceOptions {
    student: String,
    normal: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Menu {
    price: Price,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MenuList {
    data: Vec<Menu>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Restaurant {
    id: i32,
    title: String,
    slug: String,
    permalink: String,
    address: String,
    #[allow(non_camel_case_types)]
    menuData: MenuData,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env().auto_send();

    teloxide::commands_repl(bot, answer, Command::ty()).await;

    Ok(())

}

async fn get_restaurants() -> Result<Vec<Restaurant>, reqwest::Error> {
    let res = reqwest::get(UNICAFE_BASE_URL).await?;

    let restaurants: Vec<Restaurant> = res.json().await?;
    Ok(restaurants)
}

fn get_index_by_name(name: &str) -> usize {
    match name {
        "physicum" => 9,
        "exactum" => 15,
        "chemicum" => 16,
        _ => 0
    }
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "Chemicum")]
    Chemicum,
    #[command(description = "Exactum")]
    Exactum,
    #[command(description = "Physicum")]
    Physicum,
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?
        }
        Command::Username(username) => {
            bot.send_message(message.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                message.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
        Command::Chemicum => {
            let data = get_restaurants().await?;
            let index = get_index_by_name("chemicum");
            let res = format_message(&data[index].menuData.menus[0]);
            bot.send_message(message.chat.id, format!("{}", res)).parse_mode(ParseMode::MarkdownV2).await?
        }
        Command::Exactum => {
            let data = get_restaurants().await?;
            let index = get_index_by_name("exactum");
            let res = format_message(&data[index].menuData.menus[0]);
            bot.send_message(message.chat.id, format!("{}", res)).parse_mode(ParseMode::MarkdownV2).await?
        }
        Command::Physicum => {
            let data = get_restaurants().await?;
            let index = get_index_by_name("physicum");
            let res = format_message(&data[index].menuData.menus[0]);
            bot.send_message(message.chat.id, format!("{}", res)).parse_mode(ParseMode::MarkdownV2).await?
        }
    };

    Ok(())
}

fn format_message(data: &MenuList) -> String {
    let mut result = String::new();
    for menu in data.data.iter() {
        result.push_str(&format_food_and_price(menu));
    }
    if result.len() == 0 {
        result.push_str("Ei ruokaa");
    }
    format_for_markdown(result)
}

fn format_for_markdown(message: String) -> String {
    message
        .replace("-", "\\-")
        .replace(".", "\\.")
        .replace("_", "\\_")
        .replace("*", "\\*")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("(", "\\(")
        .replace(")", "\\)")
        .replace("~", "\\~")
        .replace("`", "\\`")
        .replace(">", "\\>")
        .replace("#", "\\#")
        .replace("+", "\\+")
        .replace("=", "\\=")
        .replace("|", "\\|")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("!", "\\!")
}

fn format_food_and_price(data: &Menu) -> String {
    if data.name.starts_with("RAVINTOLA AVOINNA") {
        let times = data.name.split_whitespace();
        let bolded = markdown::bold("Aukioloajat: ");
        return format!("{}{}\n\n", bolded, times.last().unwrap());
    }
    format!("{}, {}€\n", data.name, data.price.value.student)
}