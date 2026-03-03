use fluxer_builders::MessagePayload;
use fluxer_core::structures::message::Message as CoreMessage;
use fluxer_rest::Rest;
use fluxer_types::{Routes, guild::ApiGuild};

use crate::shared::{Message, embed_error};

pub async fn check_admin(rest: &Rest, message: &Message) -> bool {
    let guild_id = match &message.guild_id {
        Some(id) => id,
        None => return false,
    };

    if let Ok(guild) = rest.get::<ApiGuild>(&Routes::guild(guild_id)).await
        && guild.owner_id == message.author.id
    {
        return true;
    }

    if let Ok(member) = rest
        .get::<serde_json::Value>(&Routes::guild_member(guild_id, &message.author.id))
        .await
    {
        let member_roles: Vec<&str> = member
            .get("roles")
            .and_then(|r| r.as_array())
            .map(|r| r.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        if let Ok(roles) = rest
            .get::<Vec<serde_json::Value>>(&Routes::guild_roles(guild_id))
            .await
        {
            for role in roles {
                if let Some(id) = role.get("id").and_then(|i| i.as_str())
                    && member_roles.contains(&id)
                    && let Some(perms_str) = role.get("permissions").and_then(|p| p.as_str())
                    && let Ok(perms) = perms_str.parse::<u64>()
                {
                    // 8 is ADMINISTRATOR, 32 is MANAGE_GUILD
                    if (perms & 8) == 8
                        || (perms & 32) == 32
                        || (perms & 4) == 4
                        || (perms & 2) == 2
                    {
                        return true;
                    }
                }
            }
        }
    }

    let e = embed_error("Access Denied")
        .description("You need Administrator permissions to use this command.")
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    send_mod_reply(rest, message, &payload, false).await;
    false
}

pub async fn send_mod_reply(
    rest: &Rest,
    command_msg: &Message,
    payload: &fluxer_builders::message::MessagePayloadData,
    success: bool,
) {
    if success {
        let _ = command_msg.add_reaction(rest, "✅").await;
    }
    if let Ok(sent_api) = command_msg.send(rest, payload).await {
        let sent = CoreMessage::from_api(&sent_api);
        let rest_clone = rest.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            let _ = sent.delete(&rest_clone).await;
        });
    }
}

pub fn time_from_secs(secs: u64) -> String {
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let mut y = 1970u64;
    let mut d = days_since_epoch;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let months = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1u64;
    for &dim in &months {
        if d < dim {
            break;
        }
        d -= dim;
        mo += 1;
    }
    let day = d + 1;

    format!("{:02}.{:02}.{} {:02}:{:02}:{:02} UTC", day, mo, y, h, m, s)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

pub fn iso_8601(secs_since_epoch: u64) -> String {
    let days_since_epoch = secs_since_epoch / 86400;
    let time_of_day = secs_since_epoch % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let mut y = 1970u64;
    let mut d = days_since_epoch;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let months = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1u64;
    for &dim in &months {
        if d < dim {
            break;
        }
        d -= dim;
        mo += 1;
    }
    let day = d + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, day, h, m, s)
}

pub fn parse_duration(s: &str) -> Option<u64> {
    let mut num_str = String::new();
    let mut unit_str = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else {
            unit_str.push(c);
        }
    }
    let val: u64 = num_str.parse().ok()?;
    match unit_str.as_str() {
        "s" | "sec" | "secs" => Some(val),
        "m" | "min" | "mins" => Some(val * 60),
        "h" | "hr" | "hrs" => Some(val * 3600),
        "d" | "day" | "days" => Some(val * 86400),
        "" => Some(val),
        _ => None,
    }
}
