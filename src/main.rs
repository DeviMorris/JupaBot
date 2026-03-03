mod commands;
mod db;
mod shared;
mod util;

use fluxer_core::client::typed_events::DispatchEvent;
use fluxer_core::client::{Client, ClientOptions};
use fluxer_rest::Rest;

use shared::{PREFIX, TOKEN};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let mut client = Client::new(ClientOptions {
        intents: 1 | 2 | 512 | 1024 | 32768,
        wait_for_guilds: true,
        ..Default::default()
    });

    let rest: Rest = client.rest.clone();
    let pb_client = db::PbClient::new("http://127.0.0.1:8090");

    let rest_temp = Rest::new(fluxer_rest::RestOptions::default());
    rest_temp.set_token(TOKEN).await;
    let bot_id = if let Ok(u) = rest_temp
        .get::<serde_json::Value>(fluxer_types::Routes::current_user())
        .await
    {
        u["id"].as_str().unwrap_or("").to_string()
    } else {
        "".to_string()
    };

    client.on_typed(move |event| {
        let rest = rest.clone();
        let db_client = pb_client.clone();
        let bot_id = bot_id.clone();
        Box::pin(async move {
            match event {
                DispatchEvent::Ready => {
                    tracing::info!("Ready");
                }

                DispatchEvent::MessageCreate { message, .. } => {
                    let u_id = message.author.id.clone();

                    if message.guild_id.is_none() {
                        let captcha_state = {
                            let lock = crate::shared::CAPTCHA_STATES.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
                            let map = lock.read().unwrap();
                            map.get(&u_id).cloned()
                        };

                        if let Some(state) = captcha_state {
                            let content = message.content.trim().to_lowercase();
                            if content == state.answer {
                                let role_ids: Vec<&str> = state.roles.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                                for r_id in role_ids {
                                    let route = format!("/guilds/{}/members/{}/roles/{}", state.guild_id, u_id, r_id);
                                    let _ = rest.put_empty(&route).await;
                                }

                                {
                                    let lock = crate::shared::CAPTCHA_STATES.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
                                    let mut map = lock.write().unwrap();
                                    map.remove(&u_id);
                                }

                                let embed = crate::shared::embed_success("Success")
                                    .description("<:online:1478144903587734729> CAPTCHA solved! Roles have been assigned.")
                                    .build();
                                let payload = fluxer_builders::MessagePayload::new().add_embed(embed).build();
                                let _ = message.send(&rest, &payload).await;
                            } else {
                                let embed = crate::shared::embed_error("Error")
                                    .description("<:offline:1478144899431179463> Incorrect CAPTCHA. Please try again or re-click the reaction.")
                                    .build();
                                let payload = fluxer_builders::MessagePayload::new().add_embed(embed).build();
                                let _ = message.send(&rest, &payload).await;
                            }
                            return;
                        }
                    }

                    let content = message.content.trim();

                    if !content.starts_with(PREFIX) {
                        return;
                    }

                    let input = content.trim_start_matches(PREFIX);
                    let mut parts = input.split_whitespace();
                    let cmd = parts.next().unwrap_or("");
                    let args: Vec<&str> = parts.collect();

                    match cmd {
                        "ping" => commands::ping(&rest, &message, &args).await,
                        "help" => commands::help(&rest, &message, &args).await,
                        "dog" => commands::dog(&rest, &message, &args).await,
                        "cat" => commands::cat(&rest, &message, &args).await,
                        "fox" => commands::fox(&rest, &message, &args).await,
                        "duck" => commands::duck(&rest, &message, &args).await,
                        "avatar" => commands::avatar(&rest, &message, &args).await,
                        "serverinfo" => commands::serverinfo(&rest, &message, &args).await,
                        "clear" => commands::clear(&rest, &message, &args).await,
                        "ban" => commands::ban(&rest, &message, &args).await,
                        "unban" => commands::unban(&rest, &message, &args).await,
                        "kick" => commands::kick(&rest, &message, &args).await,
                        "mute" => commands::mute(&rest, &message, &args).await,
                        "unmute" => commands::unmute(&rest, &message, &args).await,
                        "welcome" => commands::welcome(&rest, &message, &args, &db_client).await,
                        "greet" | "greeting" => commands::greet(&rest, &message, &args, &db_client).await,
                        _ => {}
                    }
                }
                DispatchEvent::MessageReactionAdd { reaction } => {
                    let u_id = reaction.user_id.to_string();
                    if u_id == bot_id { return; }

                    let msg_id = reaction.message_id.to_string();

                    if let Some(welcome_rec) = db_client.get_welcome_message(&msg_id).await {
                        if let Some(g_id) = &reaction.guild_id {
                            let u_id = reaction.user_id.to_string();
                            let role_ids: Vec<&str> = welcome_rec.roles.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

                            let mut has_roles = false;
                            let member_route = format!("/guilds/{}/members/{}", g_id, u_id);
                            if let Ok(m_data) = rest.get::<serde_json::Value>(&member_route).await {
                                if let Some(roles_arr) = m_data["roles"].as_array() {
                                    let member_roles: Vec<String> = roles_arr.iter().filter_map(|r| r.as_str().map(String::from)).collect();
                                    for req_role in &role_ids {
                                        if member_roles.contains(&req_role.to_string()) {
                                            has_roles = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            if has_roles {
                                return;
                            }

                            if welcome_rec.captcha == "true" {
                                let cap_len = welcome_rec.captcha_len.unwrap_or(5).clamp(3, 6);
                                let body = serde_json::json!({
                                    "length": cap_len,
                                    "charset": welcome_rec.captcha_type
                                });
                                let c = reqwest::Client::new();
                                if let Ok(res) = c.post("https://api.devimorris.tech/api/captcha/generate").json(&body).send().await {
                                    if let Ok(json) = res.json::<serde_json::Value>().await {
                                        if let (Some(text), Some(b64)) = (json["captcha_text"].as_str(), json["image_base64"].as_str()) {
                                            {
                                                let state = crate::shared::CaptchaState {
                                                    roles: welcome_rec.roles.clone(),
                                                    guild_id: g_id.clone(),
                                                    answer: text.to_lowercase(),
                                                };
                                                let lock = crate::shared::CAPTCHA_STATES.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
                                                let mut map = lock.write().unwrap();
                                                map.insert(u_id.clone(), state);
                                            }

                                            // Send DM
                                            let clean_b64 = b64.replace("data:image/png;base64,", "");
                                            let img_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, clean_b64).unwrap_or_default();

                                            let dm_route = format!("/users/@me/channels");
                                            let dm_body = serde_json::json!({"recipient_id": u_id});
                                            if let Ok(dm_res) = rest.post::<serde_json::Value>(&dm_route, Some(&dm_body)).await {
                                                if let Some(dm_id) = dm_res["id"].as_str() {
                                                    let file = fluxer_builders::file::FileAttachment::new("captcha.png", img_data);
                                                    let payload = fluxer_builders::MessagePayload::new().content("Please solve this CAPTCHA by replying with the text in the image.").build();
                                                    let form = fluxer_builders::build_multipart_form(&payload, &[file]);

                                                    let route = fluxer_types::Routes::channel_messages(dm_id);
                                                    if let Err(e) = rest.post_multipart::<serde_json::Value>(&route, form).await {
                                                        tracing::error!("Failed to send CAPTCHA DM: {:?}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                for r_id in role_ids {
                                    let route = format!("/guilds/{}/members/{}/roles/{}", g_id, u_id, r_id);
                                    match rest.put_empty(&route).await {
                                        Ok(_) => tracing::info!("Gave role {} to user {}", r_id, u_id),
                                        Err(err) => tracing::error!("Failed to give role: {:?}", err),
                                    }
                                }
                            }
                        }
                    }
                }
                DispatchEvent::GuildMemberAdd { member } => {
                    let guild_id = member.guild_id.to_string();
                    if let Some(greet_rec) = db_client.get_greeting(&guild_id).await {
                        if greet_rec.enabled == "true" {
                            let channel_id = greet_rec.channel_id.clone();
                            let username = member.user.username.clone();
                            let nick = member.nick.clone().unwrap_or_else(|| member.user.display_name().to_string());

                            let avatar_url = if let Some(avatar_hash) = &member.user.avatar {
                                format!("https://fluxerusercontent.com/avatars/{}/{}.png?size=256", member.user.id, avatar_hash)
                            } else {
                                format!("https://fluxerusercontent.com/embed/avatars/{}.png", member.user.id.parse::<u64>().unwrap_or(0) % 5)
                            };

                            let body = serde_json::json!({
                                "image": avatar_url,
                                "name": nick,
                                "username": "@".to_string() + &username
                            });

                            let client = reqwest::Client::new();
                            if let Ok(res) = client.post("https://api.devimorris.tech/api/welcome").json(&body).send().await {
                                let status = res.status();
                                if status.is_success() {
                                    if let Ok(bytes) = res.bytes().await {
                                        let img_data = if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                                            if let Some(b64) = json["image_base64"].as_str() {
                                                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64.replace("data:image/png;base64,", "")).unwrap_or_default()
                                            } else {
                                                bytes.to_vec()
                                            }
                                        } else {
                                            bytes.to_vec()
                                        };


                                        if !img_data.is_empty() {
                                            let mut msg_content = format!("<@{}>", member.user.id);
                                            if let Some(mut t) = greet_rec.text {
                                                if !t.is_empty() {
                                                    t = t.replace("{ping}", &format!("<@{}>", member.user.id));
                                                    msg_content = t;
                                                }
                                            }

                                            let file = fluxer_builders::file::FileAttachment::new("welcome.png", img_data);
                                            let payload = fluxer_builders::MessagePayload::new()
                                                .content(msg_content)
                                                .build();
                                            let form = fluxer_builders::build_multipart_form(&payload, &[file]);

                                            let route = fluxer_types::Routes::channel_messages(&channel_id);
                                            if let Err(e) = rest.post_multipart::<serde_json::Value>(&route, form).await {
                                                tracing::error!("Failed to send welcome image to Fluxer: {:?}", e);
                                            }
                                        } else {
                                            tracing::error!("Welcome image data was empty");
                                        }
                                    } else {
                                        tracing::error!("Failed to read welcome API response bytes");
                                    }
                                } else {
                                    let txt = res.text().await.unwrap_or_default();
                                    tracing::error!("Welcome API returned error {}: {}", status, txt);
                                }
                            } else {
                                tracing::error!("Failed to send request to Welcome API");
                            }
                        }
                    }
                }
                _ => {}
            }
        })
    });

    if let Err(err) = client.login(TOKEN).await {
        tracing::error!("login: {err:?}");
    }
}
