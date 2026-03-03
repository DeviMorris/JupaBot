use std::time::Instant;

use fluxer_builders::MessagePayload;
use fluxer_core::structures::channel::Channel;
use fluxer_core::structures::guild::Guild;
use fluxer_core::structures::message::Message as CoreMessage;
use fluxer_core::util::cdn::CdnOptions;
use fluxer_rest::Rest;
use fluxer_types::{Routes, guild::ApiGuild, snowflake_timestamp};

use crate::shared::{Message, embed, embed_error, embed_image, embed_success, extract_id};
use crate::util::*;

pub async fn ping(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        let e = embed("!ping")
            .description("Checks if the bot is online and measures response latency.")
            .field("Usage", "`!ping`", false)
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let start = Instant::now();
    let placeholder = MessagePayload::new().content("\u{200b}").build();
    let sent_api = match message.send(rest, &placeholder).await {
        Ok(m) => m,
        Err(err) => {
            tracing::error!("ping probe: {err}");
            return;
        }
    };

    let ms = start.elapsed().as_millis();
    let edit_payload = MessagePayload::new()
        .add_embed(
            embed("Pong! 🏓")
                .field("Latency", format!("{ms} ms"), true)
                .build(),
        )
        .build();

    let sent = CoreMessage::from_api(&sent_api);
    if let Err(err) = sent.edit(rest, &edit_payload).await {
        tracing::error!("ping edit: {err}");
    }
}

pub async fn help(rest: &Rest, message: &Message, args: &[&str]) {
    if let Some(&cmd) = args.first() {
        let e = match cmd {
            "ping" => embed("!ping")
                .description("Checks if the bot is online and measures response latency.")
                .field("Usage", "`!ping`", false),
            "dog" => embed("!dog")
                .description("Shows a random dog image.")
                .field("Usage", "`!dog`", false),
            "cat" => embed("!cat")
                .description("Shows a random cat image.")
                .field("Usage", "`!cat`", false),
            "fox" => embed("!fox")
                .description("Shows a random fox image.")
                .field("Usage", "`!fox`", false),
            "duck" => embed("!duck")
                .description("Shows a random duck image.")
                .field("Usage", "`!duck`", false),
            "avatar" => embed("!avatar")
                .description("Shows your avatar or the avatar of a mentioned user.")
                .field("Usage", "`!avatar` or `!avatar @user`", false),
            "help" => embed("!help")
                .description("Lists all available commands. Pass a command name to get details.")
                .field("Usage", "`!help` or `!help <command>`", false),
            "ban" => embed("Command \"ban\"")
                .description(
                    "**Ban member from server**\n\
                     Bans specified member from server permanently or temporary and clears his messages (up to 7 days)."
                )
                .field("Usage", "`!ban <@Member | ID> [duration] [days to clear] [reason]`\n┗ Member parameter may be replaced with the author of the replied message.", false)
                .field("Example 1", "`!ban @Member`\n┗ Bans member permanently.", false)
                .field("Example 2", "`!ban @Member behaves provocatively`\n┗ Bans member permanently with specified reason.", false)
                .field("Example 3", "`!ban @Member 7 behaves provocatively`\n┗ Bans member permanently with specified reason and clears his messages for 7 last days.", false)
                .field("Example 4", "`!ban @Member 1d 7 behaves provocatively`\n┗ Bans member for one day with reason and clears his messages for 7 last days.", false),
            "unban" => embed("Command \"unban\"")
                .description(
                    "**Unban member from server**\n\
                     Unbans specified member from server by its ID, username or tag."
                )
                .field("Usage", "`!unban <ID | Username | Tag>`", false)
                .field("Example 1", "`!unban 247734710682255361`\n┗ Unbans member by ID.", false)
                .field("Example 2", "`!unban Caramel`\n┗ Unbans member by username by first match.", false)
                .field("Example 3", "`!unban caramel.foxpaws`\n┗ Unbans member by full tag.", false),
            "kick" => embed("Command \"kick\"")
                .description(
                    "**Kick member from server**\n\
                     Kicks specified member from server."
                )
                .field("Usage", "`!kick <@Member | ID> [reason]`\n┗ Member parameter may be replaced with the author of the replied message.", false)
                .field("Example 1", "`!kick @Member`\n┗ Kicks specified member.", false)
                .field("Example 2", "`!kick @Member behaves provocatively`\n┗ Kicks specified member with reason.", false),
            "mute" => embed("Command \"mute\"")
                .description(
                    "**Mute member in channel or globally on the whole server**\n\
                     Mutes specified member in channel or on the whole server permanently or temporary."
                )
                .field("Usage", "`!mute <@Member | ID> [duration] [everywhere | here | #Channel] [reason]`\n┗ Member parameter may be replaced with the author of the replied message.", false)
                .field("Example 1", "`!mute @Member`\n┗ Mutes specified member permanently in current text channel or on the whole server (depending on default mute mode).", false)
                .field("Example 2", "`!mute @Member 10min here`\n┗ Mutes specified member in current text channel for 10 minutes.", false)
                .field("Example 3", "`!mute @Member 10min everywhere`\n┗ Mutes specified member everywhere for 10 minutes.", false)
                .field("Example 4", "`!mute @Member 10min everywhere Provoker!`\n┗ Mutes specified member everywhere for 10 minutes and sets mute reason.", false)
                .field("Example 5", "`!mute @Member 10min #Channel Provoker!`\n┗ Mutes specified member in specified channel for 10 minutes and sets mute reason.", false),
            "unmute" => embed("Command \"unmute\"")
                .description(
                    "**Unmute member in current channel or globally**\n\
                     Removes all restrictions done by \"!mute\" command in current channel or globally."
                )
                .field("Usage", "`!unmute <@Member | ID> [#Channel | ID]`\n┗ Member parameter may be replaced with the author of the replied message.", false)
                .field("Example 1", "`!unmute @Member`\n┗ Unmutes specified member in current channel or globally.", false)
                .field("Example 2", "`!unmute @Member #Channel`\n┗ Unmutes specified member in specified channel or globally.", false)
                .field("Example 3", "`!unmute @Member 123456789012345678`\n┗ Unmutes specified member by ID in the specified channel.", false),
            "welcome" => embed("Command \"welcome\"")
                .description(
                    "**Send a welcome message**\n\
                     Sends a custom welcome message to a specified channel with a predefined reaction."
                )
                .field("Usage", "`!welcome [#channel] title: [Text] text: [Text]`", false)
                .field("Example", "`!welcome #general title: Welcome! text: Hello, new members!`\nIf channel is omitted, sends to the current channel.", false),
            _ => embed("Unknown command")
                .description(format!("`!{cmd}` is not a known command.")),
        };

        let payload = MessagePayload::new().add_embed(e.build()).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let e = embed("Available commands")
        .description(
            "Pass a command name to get details — `!help ping` — or add `?` after any command.\n\
             Example: `!ping ?`",
        )
        .field(
            "📋  Information",
            "`!help`  `!ping`  `!avatar`  `!serverinfo`",
            false,
        )
        .field(
            "🛡️  Moderation",
            "`!clear`  `!ban`  `!unban`  `!kick`  `!mute`  `!unmute`  `!welcome`",
            false,
        )
        .field("😄  Fun", "`!dog`  `!cat`  `!fox`  `!duck`", false)
        .build();

    let payload = MessagePayload::new().add_embed(e).build();
    if let Err(err) = message.send(rest, &payload).await {
        tracing::error!("help: {err}");
    }
}

pub async fn avatar(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        let e = embed("!avatar")
            .description("Shows your avatar or the avatar of a mentioned user.")
            .field("Usage", "`!avatar` or `!avatar @user`", false)
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let cdn = CdnOptions::default();
    let target = message.mentions.first().unwrap_or(&message.author);
    let url = target.display_avatar_url(&cdn);

    let e = embed_image(&format!("{}'s avatar", target.username), &url).build();

    let payload = MessagePayload::new().add_embed(e).build();
    if let Err(err) = message.send(rest, &payload).await {
        tracing::error!("avatar: {err}");
    }
}

pub async fn serverinfo(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        let e = embed("!serverinfo")
            .description("Shows information about the current server.")
            .field("Usage", "`!serverinfo`", false)
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let payload = MessagePayload::new()
                .content("This command only works in a server.")
                .build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let api_guild: ApiGuild = match rest.get(&Routes::guild(&guild_id)).await {
        Ok(g) => g,
        Err(err) => {
            tracing::error!("serverinfo fetch guild: {err}");
            return;
        }
    };

    let guild = Guild::from_api(&api_guild);

    let members = guild
        .fetch_members(rest, Some(1000), None)
        .await
        .unwrap_or_default();
    let total = members.len();
    let bots = members
        .iter()
        .filter(|m| m.user.as_ref().and_then(|u| u.bot) == Some(true))
        .count();
    let humans = total - bots;

    let created = snowflake_timestamp(&guild.id)
        .map(|ms| {
            let secs = ms / 1000;

            time_from_secs(secs)
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let cdn = CdnOptions::default();
    let icon = guild.icon_url(&cdn).unwrap_or_default();

    let e = embed(&format!("Information about {}", guild.name))
        .thumbnail(&icon)
        .field("Guild ID", format!("`{}`", guild.id), false)
        .field(
            "**Members:**",
            format!(
                "<:people:1478146494952821960> Total: **{total}**\n\
                 <:member:1478148900809766436> Members: {humans}\n\
                 <:bot:1478148899341760035> Bots: {bots}"
            ),
            false,
        )
        .field("<:online:1478144903587734729>  Created", created, false)
        .footer("JupaBot", Some(crate::shared::LOGO.to_string()))
        .build();

    let payload = MessagePayload::new().add_embed(e).build();
    if let Err(err) = message.send(rest, &payload).await {
        tracing::error!("serverinfo: {err}");
    }
}

pub async fn clear(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    let n: usize = match args.first().and_then(|s| s.parse().ok()) {
        Some(n) if (1..=100).contains(&n) => n,
        _ => {
            let e = embed_error("Invalid usage")
                .description("Usage: `!clean <1-100>`")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let channel = Channel::from_id(&message.channel_id);

    let messages = match channel.fetch_messages(rest, Some(100), None, None).await {
        Ok(m) => m,
        Err(err) => {
            tracing::error!("clean fetch: {err}");
            return;
        }
    };

    let ids: Vec<String> = messages.into_iter().take(n).map(|m| m.id).collect();
    let deleted = ids.len();

    if ids.is_empty() {
        let e = embed_error("Nothing to delete").build();
        let payload = MessagePayload::new().add_embed(e).build();
        send_mod_reply(rest, message, &payload, false).await;
        return;
    }

    if let Err(err) = channel.bulk_delete_messages(rest, &ids).await {
        tracing::error!("clean bulk_delete: {err}");
        let e = embed_error("Failed")
            .description("Missing permissions or messages are too old.")
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        send_mod_reply(rest, message, &payload, false).await;
        return;
    }

    let e = embed_success("Done")
        .description(format!("Deleted **{deleted}** messages."))
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    send_mod_reply(rest, message, &payload, true).await;
}

pub async fn ban(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    if args.is_empty() || args.first() == Some(&"?") {
        help(rest, message, &["ban"]).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let e = embed_error("Error")
                .description("This command can only be used in a server.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let target_id = match extract_id(args[0]) {
        Some(id) => id,
        None => {
            let e = embed_error("Invalid Target")
                .description("Please provide a valid member mention or ID.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    let guild = Guild::from_id(&guild_id);
    match guild.ban(rest, &target_id, reason.as_deref()).await {
        Ok(_) => {
            let desc = if let Some(r) = reason {
                format!("Banned <@{target_id}>.\n**Reason:** {r}")
            } else {
                format!("Banned <@{target_id}>.")
            };
            let e = embed_success("Member Banned").description(desc).build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("ban failed: {err}");
            let e = embed_error("Failed to Ban").description("Make sure the bot has permissions and the user's role is lower than the bot's.").build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn unban(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    if args.is_empty() || args.first() == Some(&"?") {
        help(rest, message, &["unban"]).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let e = embed_error("Error")
                .description("This command can only be used in a server.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let target_id = match extract_id(args[0]) {
        Some(id) => id,
        None => {
            let e = embed_error("Invalid Target")
                .description("Please provide a valid user ID or username.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let guild = Guild::from_id(&guild_id);
    match guild.unban(rest, &target_id).await {
        Ok(_) => {
            let e = embed_success("Member Unbanned")
                .description(format!("Unbanned <@{target_id}>."))
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("unban failed: {err}");
            let e = embed_error("Failed to Unban")
                .description("Make sure the bot has permissions or the user is actually banned.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn kick(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    if args.is_empty() || args.first() == Some(&"?") {
        help(rest, message, &["kick"]).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let e = embed_error("Error")
                .description("This command can only be used in a server.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let target_id = match extract_id(args[0]) {
        Some(id) => id,
        None => {
            let e = embed_error("Invalid Target")
                .description("Please provide a valid member mention or ID.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            let _ = message.send(rest, &payload).await;
            return;
        }
    };

    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    let guild = Guild::from_id(&guild_id);
    match guild.kick(rest, &target_id).await {
        Ok(_) => {
            let desc = if let Some(r) = reason {
                format!("Kicked <@{target_id}>.\n**Reason:** {r}")
            } else {
                format!("Kicked <@{target_id}>.")
            };
            let e = embed_success("Member Kicked").description(desc).build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("kick failed: {err}");
            let e = embed_error("Failed to Kick").description("Make sure the bot has permissions and the user's role is lower than the bot's.").build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn mute(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    if args.is_empty() || args.first() == Some(&"?") {
        help(rest, message, &["mute"]).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let e = embed_error("Error")
                .description("This command can only be used in a server.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
            return;
        }
    };

    let target_id = match extract_id(args[0]) {
        Some(id) => id,
        None => {
            let e = embed_error("Invalid Target")
                .description("Please provide a valid member mention or ID.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
            return;
        }
    };

    let duration_secs = args.get(1).and_then(|s| parse_duration(s));
    let mut reason = None;
    if args.len() > 1 {
        let start_idx = if duration_secs.is_some() { 2 } else { 1 };
        if args.len() > start_idx {
            reason = Some(args[start_idx..].join(" "));
        }
    }

    let guild = Guild::from_id(&guild_id);
    let mut req = serde_json::Map::new();

    if let Some(secs) = duration_secs {
        let until = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + secs;
        req.insert(
            "communication_disabled_until".to_string(),
            serde_json::Value::String(iso_8601(until)),
        );
    } else {
        let until = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + (28 * 86400);
        req.insert(
            "communication_disabled_until".to_string(),
            serde_json::Value::String(iso_8601(until)),
        );
    }

    let body = serde_json::Value::Object(req);
    match rest
        .patch::<serde_json::Value>(&Routes::guild_member(&guild.id, &target_id), Some(&body))
        .await
    {
        Ok(_) => {
            let desc = if let Some(r) = &reason {
                format!("Muted <@{target_id}>.\n**Reason:** {r}")
            } else {
                format!("Muted <@{target_id}>.")
            };
            let e = embed_success("Member Muted").description(desc).build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("mute failed: {err}");
            let e = embed_error("Failed to Mute")
                .description("Make sure the bot has permissions (Timeout Members).")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn unmute(rest: &Rest, message: &Message, args: &[&str]) {
    if !check_admin(rest, message).await {
        return;
    }
    if args.is_empty() || args.first() == Some(&"?") {
        help(rest, message, &["unmute"]).await;
        return;
    }

    let guild_id = match &message.guild_id {
        Some(id) => id.clone(),
        None => {
            let e = embed_error("Error")
                .description("This command can only be used in a server.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
            return;
        }
    };

    let target_id = match extract_id(args[0]) {
        Some(id) => id,
        None => {
            let e = embed_error("Invalid Target")
                .description("Please provide a valid member mention or ID.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
            return;
        }
    };

    let guild = Guild::from_id(&guild_id);
    let mut req = serde_json::Map::new();
    req.insert(
        "communication_disabled_until".to_string(),
        serde_json::Value::Null,
    );
    let body = serde_json::Value::Object(req);

    match rest
        .patch::<serde_json::Value>(&Routes::guild_member(&guild.id, &target_id), Some(&body))
        .await
    {
        Ok(_) => {
            let e = embed_success("Member Unmuted")
                .description(format!("Unmuted <@{target_id}>."))
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("unmute failed: {err}");
            let e = embed_error("Failed to Unmute")
                .description("Make sure the bot has permissions.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn welcome(rest: &Rest, message: &Message, args: &[&str], db: &crate::db::PbClient) {
    if !check_admin(rest, message).await {
        return;
    }

    if args.is_empty() || args.first() == Some(&"?") {
        let e = embed_image("Welcome Setup Instructions", "https://devimorris.tech/example.png").description(
                "Creates a welcome message with an automatic role given upon reacting.\n\
                **Format:** `!welcome #channel title: <title> text: <text> reaction: <emoji> start_roles: <role1>, <role2> captcha: <true/false> captcha_type: <type> captcha_len: <3-6>`\n\n\
                **Parameters:**\n\
                `#channel` - The channel to send the message (as a mention, must be the first argument).\n\
                `title:` - Title of the embed.\n\
                `text:` - Main content of the embed.\n\
                `reaction:` - Emoji to add to the message for users to click.\n\
                `start_roles:` - Comma separated roles (mentions or IDs) to give.\n\
                `captcha:` - (Optional, default `false`) If `true`, a CAPTCHA will be sent via DM before giving roles.\n\
                `captcha_type:` - (Optional, default `letters_digits`) The character set for the CAPTCHA. Available types: `digits`, `letters`, `special`, `letters_digits`, `full`.\n\
                `captcha_len:` - (Optional, default `5`) The length of the CAPTCHA string (between `3` and `6`).\n\
                \n\
                **Example:**\n\
                `!welcome #general title: Welcome! text: Click to get access! reaction: ✅ start_roles: @Member captcha: true captcha_type: letters_digits captcha_len: 5`"
            ).image("https://devimorris.tech/example.png").build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let full_text = &message.content;

    let mut target_channel = message.channel_id.clone();

    if let Some(first) = args.first() {
        if first.starts_with("<#") && first.ends_with('>') {
            target_channel = first[2..first.len() - 1].to_string();
        } else if first.chars().all(|c| c.is_ascii_digit()) && !first.is_empty() {
            target_channel = first.to_string();
        }
    }

    let mut parts: Vec<(usize, &str)> = Vec::new();
    if let Some(i) = full_text.find("title:") {
        parts.push((i, "title"));
    }
    if let Some(i) = full_text.find("text:") {
        parts.push((i, "text"));
    }
    if let Some(i) = full_text.find("reaction:") {
        parts.push((i, "reaction"));
    }
    if let Some(i) = full_text.find("start_roles:") {
        parts.push((i, "start_roles"));
    }
    if let Some(i) = full_text.find("captcha:") {
        parts.push((i, "captcha"));
    }
    if let Some(i) = full_text.find("captcha_type:") {
        parts.push((i, "captcha_type"));
    }
    if let Some(i) = full_text.find("captcha_len:") {
        parts.push((i, "captcha_len"));
    }

    if !parts.iter().any(|p| p.1 == "title") || !parts.iter().any(|p| p.1 == "text") {
        let e = embed_error("Invalid Format")
            .description("Please provide both `title:` and `text:` parameters.\nExample: `!welcome #channel title: Hello! text: Welcome here! reaction: 😃 start_roles: @Role1 captcha: true captcha_type: letters_digits captcha_len: 5`")
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        send_mod_reply(rest, message, &payload, false).await;
        return;
    }

    parts.sort_by_key(|p| p.0);

    let mut title = "";
    let mut text = "";
    let mut reaction = None;
    let mut start_roles = "";
    let mut captcha = "false";
    let mut captcha_type = "letters_digits";
    let mut captcha_len_str = "5";

    for i in 0..parts.len() {
        let start = parts[i].0 + parts[i].1.len() + 1;
        let end = if i + 1 < parts.len() {
            parts[i + 1].0
        } else {
            full_text.len()
        };
        let content = full_text[start..end].trim();
        match parts[i].1 {
            "title" => title = content,
            "text" => text = content,
            "reaction" => reaction = Some(content),
            "start_roles" => start_roles = content,
            "captcha" => captcha = content,
            "captcha_type" => captcha_type = content,
            "captcha_len" => captcha_len_str = content,
            _ => {}
        }
    }

    let captcha_len: u32 = captcha_len_str.parse().unwrap_or(5).clamp(3, 6);

    if title.is_empty() || text.is_empty() {
        let e = embed_error("Empty Content")
            .description("Title and text cannot be empty.")
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        send_mod_reply(rest, message, &payload, false).await;
        return;
    }

    let welcome_embed = crate::shared::embed(title).description(text).build();
    let welcome_payload = MessagePayload::new().add_embed(welcome_embed).build();

    let _ = message.delete(rest).await;

    let channel = Channel::from_id(&target_channel);
    let guild_id = message.guild_id.clone().unwrap_or_default();

    match channel.send(rest, &welcome_payload).await {
        Ok(sent_api) => {
            let sent = CoreMessage::from_api(&sent_api);

            if let Some(emoji_input) = reaction {
                let clean_emoji = if emoji_input.starts_with("<:") && emoji_input.ends_with('>') {
                    &emoji_input[2..emoji_input.len() - 1]
                } else if emoji_input.starts_with("<a:") && emoji_input.ends_with('>') {
                    &emoji_input[3..emoji_input.len() - 1]
                } else {
                    emoji_input
                };

                if let Err(e) = sent.add_reaction(rest, clean_emoji).await {
                    tracing::error!("Failed to add custom reaction {clean_emoji}: {e}");
                }
            } else if let Err(e) = sent.add_reaction(rest, "online:1478144903587734729").await {
                tracing::error!("Failed to add reaction: {e}");
            }

            let mut role_ids = Vec::new();
            for r in start_roles.split(|c: char| c == ',' || c.is_whitespace()) {
                if let Some(id) = crate::shared::extract_id(r.trim()) {
                    role_ids.push(id.clone());
                }
            }
            let roles_str = role_ids.join(",");

            if !roles_str.is_empty()
                && let Err(e) = db
                    .save_welcome_message(
                        &guild_id,
                        &sent.id,
                        &target_channel,
                        &roles_str,
                        captcha,
                        captcha_type,
                        captcha_len,
                    )
                    .await
            {
                tracing::error!("Failed to save welcome message to PocketBase: {}", e);
            }

            let e = embed_success("Success")
                .description(format!("Welcome message sent to <#{target_channel}>."))
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, true).await;
        }
        Err(err) => {
            tracing::error!("welcome send failed: {err}");
            let e = embed_error("Failed")
                .description("Could not send the message. Make sure the bot has permissions in that channel.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}

pub async fn cat(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        help(rest, message, &["cat"]).await;
        return;
    }

    if let Ok(resp) = reqwest::get("https://cataas.com/cat?json=true").await
        && let Ok(json) = resp.json::<serde_json::Value>().await
        && let Some(url) = json.get("url").and_then(|u| u.as_str())
    {
        let image_url = if url.starts_with("http") {
            url.to_string()
        } else {
            format!("https://cataas.com{url}")
        };
        let e = embed_image("Random Cat 🐈", &image_url).build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }
    let e = embed_error("Error")
        .description("Failed to fetch cat image.")
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    let _ = message.send(rest, &payload).await;
}

pub async fn dog(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        help(rest, message, &["dog"]).await;
        return;
    }

    if let Ok(resp) = reqwest::get("https://dog.ceo/api/breeds/image/random").await
        && let Ok(json) = resp.json::<serde_json::Value>().await
        && let Some(url) = json.get("message").and_then(|u| u.as_str())
    {
        let e = embed_image("Random Dog 🐕", url).build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }
    let e = embed_error("Error")
        .description("Failed to fetch dog image.")
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    let _ = message.send(rest, &payload).await;
}

pub async fn fox(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        help(rest, message, &["fox"]).await;
        return;
    }

    if let Ok(resp) = reqwest::get("https://randomfox.ca/floof/").await
        && let Ok(json) = resp.json::<serde_json::Value>().await
        && let Some(url) = json.get("image").and_then(|u| u.as_str())
    {
        let e = embed_image("Random Fox 🦊", url).build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }
    let e = embed_error("Error")
        .description("Failed to fetch fox image.")
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    let _ = message.send(rest, &payload).await;
}

pub async fn duck(rest: &Rest, message: &Message, args: &[&str]) {
    if args.first() == Some(&"?") {
        help(rest, message, &["duck"]).await;
        return;
    }

    if let Ok(resp) = reqwest::get("https://random-d.uk/api/v2/random").await
        && let Ok(json) = resp.json::<serde_json::Value>().await
        && let Some(url) = json.get("url").and_then(|u| u.as_str())
    {
        let e = embed_image("Random Duck 🦆", url).build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }
    let e = embed_error("Error")
        .description("Failed to fetch duck image.")
        .build();
    let payload = MessagePayload::new().add_embed(e).build();
    let _ = message.send(rest, &payload).await;
}

pub async fn greet(rest: &Rest, message: &Message, args: &[&str], db: &crate::db::PbClient) {
    if !check_admin(rest, message).await {
        return;
    }

    if args.is_empty() || args.first() == Some(&"?") {
        let e = embed("Greeting Setup")
            .description(
                "Creates an automatic image greeting for new server members.\n\
                **Format:** `!greet <text> | <#channel> | <true/false>`\n\n\
                **Parameters:**\n\
                `<text>` - The text to send along with the image. You can use `{ping}` to mention the user.\n\
                `<#channel>` - The channel to send the image greeting.\n\
                `<true/false>` - Whether the greeting is enabled or not.\n\n\
                **Example:**\n\
                `!greet Welcome {ping} to the server! | #general | true`"
            )
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        let _ = message.send(rest, &payload).await;
        return;
    }

    let input = message
        .content
        .trim_start_matches("!greet")
        .trim_start_matches("!greeting")
        .trim();
    let parts: Vec<&str> = input.split('|').map(|s| s.trim()).collect();

    if parts.len() < 3 {
        let e = embed_error("Invalid Format")
            .description("Please provide text, channel, and true/false parameter separated by `|`.\nExample: `!greet Welcome {ping} | #general | true`")
            .build();
        let payload = MessagePayload::new().add_embed(e).build();
        send_mod_reply(rest, message, &payload, false).await;
        return;
    }

    let text_content = parts[0];
    let channel_mention = parts[1];
    let enabled_str = parts[2];

    let target_channel = if channel_mention.starts_with("<#") && channel_mention.ends_with('>') {
        channel_mention[2..channel_mention.len() - 1].to_string()
    } else if channel_mention.chars().all(|c| c.is_ascii_digit()) && !channel_mention.is_empty() {
        channel_mention.to_string()
    } else {
        message.channel_id.clone()
    };
    let enabled = if enabled_str == "true" {
        "true"
    } else {
        "false"
    };

    if let Some(guild_id) = &message.guild_id {
        if let Err(e) = db
            .save_greeting(guild_id, &target_channel, enabled, text_content)
            .await
        {
            tracing::error!("Failed to save greeting message to PocketBase: {}", e);
            let e = embed_error("Database Error")
                .description("Could not save greeting settings. Please try again.")
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        } else {
            let e = embed_success("Greeting Configured")
                .description(format!(
                    "Automatic greeting successfully set for <#{}>",
                    target_channel
                ))
                .build();
            let payload = MessagePayload::new().add_embed(e).build();
            send_mod_reply(rest, message, &payload, false).await;
        }
    }
}
