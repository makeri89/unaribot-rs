use chrono::offset;
use serde::{Deserialize, Serialize};
use std::error::Error;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

static UNICAFE_BASE_URL: &str = "https://unicafe.fi/wp-json/swiss/v1/restaurants/?lang=fi";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MenuData {
    name: String,
    id: i32,
    areacode: i32,
    menus: Vec<MenuList>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Price {
    name: String,
    value: PriceOptions,
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
    data: Vec<Menu>,
    date: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Restaurant {
    id: i32,
    title: String,
    slug: String,
    permalink: String,
    address: String,
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

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Chemicum menu.")]
    Chemicum,
    #[command(description = "Exactum menu")]
    Exactum,
    #[command(description = "Physicum menu")]
    Physicum,
    #[command(description = "Kaivopiha menu")]
    Kaivopiha,
    #[command(description = "WELL Kaisas menu")]
    WELLKaisa,
    #[command(description = "Viikuna menu")]
    Viikuna,
    #[command(description = "Sockom menu")]
    Sockom,
    #[command(description = "Rotunda menu")]
    Rotunda,
    #[command(description = "Porthania menu")]
    Porthania,
    #[command(description = "Topelias menu")]
    Pescovege,
    #[command(description = "Olivia menu")]
    Olivia,
    #[command(description = "Metsatalo menu")]
    Metsatalo,
    #[command(description = "Meilahti menu")]
    Meilahti,
    #[command(description = "Infokeskus menu")]
    Infokeskus,
    #[command(description = "Portaali menu")]
    Portaali,
    #[command(description = "Biokeskus 3 menu")]
    Biokeskus,
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Chemicum => message_sender("chemicum", message, bot).await?,
        Command::Exactum => message_sender("exactum", message, bot).await?,
        Command::Physicum => message_sender("physicum", message, bot).await?,
        Command::Kaivopiha => message_sender("kaivopiha", message, bot).await?,
        Command::WELLKaisa => message_sender("well-kaisa-talo", message, bot).await?,
        Command::Viikuna => message_sender("viikuna", message, bot).await?,
        Command::Sockom => message_sender("sockom", message, bot).await?,
        Command::Rotunda => message_sender("rotunda", message, bot).await?,
        Command::Porthania => message_sender("porthania", message, bot).await?,
        Command::Pescovege => message_sender("pesco-vege-topelias", message, bot).await?,
        Command::Olivia => message_sender("olivia", message, bot).await?,
        Command::Metsatalo => message_sender("metsatalo", message, bot).await?,
        Command::Meilahti => message_sender("meilahti", message, bot).await?,
        Command::Infokeskus => message_sender("infokeskus", message, bot).await?,
        Command::Portaali => message_sender("portaali", message, bot).await?,
        Command::Biokeskus => message_sender("biokeskus-3", message, bot).await?,
    };

    Ok(())
}

async fn message_sender(
    name: &str,
    message: Message,
    bot: AutoSend<Bot>,
) -> Result<teloxide::prelude::Message, Box<dyn Error + Send + Sync>> {
    let data = get_restaurants().await?;
    let restaurant_index = find_by_slug(&data, name);
    let restaurant = &data[restaurant_index];
    let today = offset::Local::today().format("%d.%m.");
    let today = format!("{}", today);
    let menu_index = get_menu_index_by_date(&restaurant.menuData, &today);
    let res = format_message(&restaurant.menuData.menus[menu_index]);
    Ok(bot
        .send_message(message.chat.id, format!("{}", res))
        .parse_mode(ParseMode::MarkdownV2)
        .await?)
}

fn find_by_slug(data: &Vec<Restaurant>, rest_slug: &str) -> usize {
    data.iter()
        .position(|x| x.slug.contains(rest_slug))
        .unwrap()
}

fn get_menu_index_by_date(data: &MenuData, date: &String) -> usize {
    data.menus
        .iter()
        .position(|x| x.date.contains(date))
        .unwrap()
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
        return format!("Aukioloajat: {}\n\n", times.last().unwrap());
    }
    format!("{}, {}€\n", data.name, data.price.value.student)
}
