use fluxer_builders::EmbedBuilder;

pub const TOKEN: &str = "TOKEN";
pub const BOT_NAME: &str = "JupaBot";
pub const COLOR: u32 = 0x9C8EE9;
pub const PREFIX: &str = "!";
pub const LOGO: &str = "https://devimorris.tech/jupa_logo.png";

pub type Message = fluxer_core::structures::message::Message;

#[derive(Clone)]
pub struct CaptchaState {
    pub roles: String,
    pub guild_id: String,
    pub answer: String,
}

pub static CAPTCHA_STATES: std::sync::OnceLock<
    std::sync::RwLock<std::collections::HashMap<String, CaptchaState>>,
> = std::sync::OnceLock::new();

pub fn embed(title: &str) -> EmbedBuilder {
    EmbedBuilder::new()
        .color(COLOR)
        .title(title)
        .thumbnail(LOGO)
        .footer(BOT_NAME, Some(LOGO.to_string()))
}

pub fn embed_image(title: &str, image: &str) -> EmbedBuilder {
    EmbedBuilder::new()
        .color(COLOR)
        .title(title)
        .image(image)
        .footer(BOT_NAME, Some(LOGO.to_string()))
}

pub fn embed_success(title: &str) -> EmbedBuilder {
    EmbedBuilder::new()
        .color(0x57F287)
        .title(title)
        .thumbnail(LOGO)
        .footer(BOT_NAME, Some(LOGO.to_string()))
}

pub fn embed_error(title: &str) -> EmbedBuilder {
    EmbedBuilder::new()
        .color(0xED4245)
        .title(title)
        .thumbnail(LOGO)
        .footer(BOT_NAME, Some(LOGO.to_string()))
}

pub fn extract_id(input: &str) -> Option<String> {
    let s = input.trim();
    if s.starts_with("<@") && s.ends_with('>') {
        let inner = &s[2..s.len() - 1];
        let id = inner.trim_start_matches('!').trim_start_matches('&');
        if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
            return Some(id.to_string());
        }
    } else if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
        return Some(s.to_string());
    }
    None
}
