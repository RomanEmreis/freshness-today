use crate::{Users, api::{fetch_air_quality, AirResponse, ApiKey}};
use std::sync::Arc;
use chrono::Local;
use reqwest::Client;
use teloxide::{
    prelude::*,
    types::{
        ButtonRequest,
        KeyboardButton,
        KeyboardMarkup,
        ParseMode
    },
};

pub(super) async fn air_quality(
    bot: Bot,
    msg: Message,
    users: Users,
    client: Arc<Client>,
    api_key: Arc<ApiKey>
) -> anyhow::Result<(), anyhow::Error> {
    let chat_id = msg.chat.id;
    let users_read = users.read().await;
    let location = match users_read.get(&chat_id.0) {
        Some(c) => *c,
        None => {
            drop(users_read);

            bot.send_message(
                chat_id,
                r#"❗ Сначала поделись местоположением"#,
            )
                .reply_markup(get_location_keyboard())
                .await?;
            return Ok(());
        }
    };
    drop(users_read);

    let resp = fetch_air_quality(&client, &api_key, &location).await?;

    bot.send_message(
        chat_id,
        format_message(resp),
    )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(get_main_keyboard())
        .await?;

    Ok(())
}

pub(super) async fn start(bot: Bot, msg: Message, users: Users) -> anyhow::Result<(), anyhow::Error> {
    let users_read = users.read().await;
    let has_location = users_read.contains_key(&msg.chat.id.0);
    drop(users_read);

    if has_location {
        bot.send_message(
            msg.chat.id,
            r#"С возвращением! Нажми кнопку, чтобы проверить качество воздуха."#)
            .reply_markup(get_main_keyboard())
            .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            r#"Привет! Отправь своё местоположение, чтобы узнать качество воздуха рядом с тобой."#)
            .reply_markup(get_location_keyboard())
            .await?;
    }

    Ok(())
}

pub(super) async fn handle_location(bot: Bot, msg: Message, users: Users) -> anyhow::Result<(), anyhow::Error> {
    if let Some(location) = msg.location() {
        let lat = location.latitude;
        let lon = location.longitude;

        users.write()
            .await
            .insert(msg.chat.id.0, (lat, lon));

        bot.send_message(
            msg.chat.id,
            format!(r#"✅ Местоположение сохранено: {lat}, {lon}"#))
            .reply_markup(get_main_keyboard())
            .await?;
    }
    Ok(())
}

fn format_message(resp: AirResponse) -> String {
    let aqi = resp.data.current.pollution.aqius;
    let status = match aqi {
        0..=50 => "🟢 Отлично",
        51..=100 => "🟡 Нормально",
        101..=150 => "🟠 Вредно для чувствительных",
        151..=200 => "🔴 Вредно",
        _ => "☠️ Очень вредно",
    };

    let time = Local::now().format("%H:%M");
    let city = resp.data.city;
    format!(
        r#"*Качество воздуха*
         🏙 Город: *{city}*
         🕒 {time}
         🌫 AQI: *{aqi}*
         📊 {status}"#
    )
}

#[inline]
fn get_location_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(r#"📍 Отправить локацию"#)
                .request(ButtonRequest::Location),
        ]
    ])
        .resize_keyboard()
}

#[inline]
fn get_main_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("🌫 Качество воздуха"),
        ]
    ])
        .resize_keyboard()
}