use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::read_to_string;
use std::sync::RwLock;
use std::time::Duration;

use anyhow::Context as _;

use poise::builtins::{register_globally, register_in_guild};
use poise::serenity_prelude::{
    ChannelId, ClientBuilder, Colour, GatewayIntents, GuildId, ReactionType, RoleId, UserId,
};
use poise::{Framework, PrefixFrameworkOptions};
use serde::Deserialize;
use shuttle_runtime::{CustomError, SecretStore};
use shuttle_serenity::ShuttleSerenity;
use sqlx::{query, PgPool};
use tracing::info;

use crate::check_reminder::check_reminders;
use crate::commands::*;
use crate::hashable_regex::HashableRegex;
use crate::role_removal::check_role_removals;

mod check_reminder;
mod commands;
mod constants;
mod easy_embed;
mod handler;
mod hashable_regex;
mod role_removal;

#[derive(Debug, Deserialize)]
struct AutoReply {
    keywords: Vec<HashableRegex>,
    user: UserId,
    title: String,
    description: String,
    #[serde(default)]
    ping: bool,
    #[serde(default)]
    /// colour as an integer
    colour: Colour,
}

#[derive(Debug, Deserialize)]
struct RoleRemoval {
    guild_id: GuildId,
    channel_id: ChannelId,
    role_id: RoleId,
    timeout_days: u64,
}

#[derive(Deserialize)]
struct Config {
    event_channel_per_guild: HashMap<GuildId, ChannelId>,
    excluded_channels: HashSet<ChannelId>,
    auto_reactions: HashMap<HashableRegex, ReactionType>,
    auto_replies: Vec<AutoReply>,
    role_removals: Vec<RoleRemoval>,
}

/// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub(crate) struct Data {
    cat_api_token: String,
    dog_api_token: String,
    mp_api_token: String,
    database: PgPool,
    event_channel_per_guild: HashMap<GuildId, ChannelId>,
    excluded_channels: RwLock<HashSet<ChannelId>>,
    auto_reactions: HashMap<HashableRegex, ReactionType>,
    auto_replies: Vec<AutoReply>,
    reaction_msgs: RwLock<HashSet<u64>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "postgres://test:pass@localhost:5432/postgres")]
    pool: PgPool,
) -> ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let config_data = read_to_string("assets/config.hjson").context("Couldn't load config file")?;
    let config: Config = deser_hjson::from_str(&config_data).context("Bad config")?;

    let commands = vec![
        activity(),
        floof(),
        boop(),
        capy(),
        clear(),
        mp(),
        cutie_pie(),
        emoji(),
        event(),
        exclude(),
        export_events(),
        features(),
        keyword_statistics(),
        latency(),
        mensa(),
        modules(),
        obama(),
        ping(),
        react(),
        reaction_role(),
        register_commands(),
        reminder(),
        say(),
        servers(),
        uwu(),
        uwu_text(),
    ];
    let framework = Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            event_handler: |ctx, event, _framework, data| {
                Box::pin(handler::event_handler(ctx, event, _framework, data))
            },
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, _framework| {
            Box::pin(async move {
                info!("{} is connected!", ready.user.name);
                register_globally(ctx, &[modules(), register_commands()]).await?;
                for guild in &ready.guilds {
                    let modules = get_active_modules(&pool, guild.id).await?;
                    register_in_guild(ctx, &get_active_commands(modules), guild.id).await?;
                    info!("Loaded modules for guild {}", guild.id);
                }
                let reaction_msgs: Vec<_> = query!("SELECT message_id FROM reaction_roles")
                    .fetch_all(&pool)
                    .await?;
                info!("Loaded reaction messages");
                check_reminders(ctx.clone(), pool.clone(), Duration::from_secs(60));
                info!("Started reminder thread");
                check_role_removals(
                    ctx.clone(),
                    config.role_removals,
                    Duration::from_secs(24 * 60 * 60),
                );
                Ok(Data {
                    cat_api_token: secret_store.get("CAT_API_TOKEN").unwrap_or("".to_string()),
                    dog_api_token: secret_store.get("DOG_API_TOKEN").unwrap_or("".to_string()),
                    mp_api_token: secret_store
                        .get("MENSAPLAN_API_TOKEN")
                        .unwrap_or("".to_string()),
                    database: pool,
                    excluded_channels: RwLock::new(config.excluded_channels),
                    event_channel_per_guild: config.event_channel_per_guild,
                    auto_reactions: config.auto_reactions,
                    auto_replies: config.auto_replies,
                    reaction_msgs: RwLock::new(
                        reaction_msgs.iter().map(|f| f.message_id as u64).collect(),
                    ),
                })
            })
        })
        .build();

    let client = ClientBuilder::new(
        discord_token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::GUILD_SCHEDULED_EVENTS,
    )
    .framework(framework)
    .await
    .map_err(CustomError::new)?;

    Ok(client.into())
}
