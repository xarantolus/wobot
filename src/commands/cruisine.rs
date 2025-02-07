use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::sync::OnceLock;

use anyhow::{anyhow, Context as _};
use chrono::{DateTime, Duration, Utc};
use image::{DynamicImage, GenericImage};
use poise::futures_util::future::try_join_all;
use poise::serenity_prelude::UserId;
use stitchy_core::Stitch;
use tracing::{debug, info};

use crate::commands::send_image;
use crate::commands::utils::load_avatar;
use crate::{Context, Error};

const DISAPPEAR_TIME: Duration = Duration::milliseconds(3600 * 1000);
const MENSA_PLAN_PATH: &str = "assets/mensa_plan.png";
static MENSA_PLAN_IMAGE: OnceLock<DynamicImage> = OnceLock::new();

const MIN_X: char = 'A';
const MAX_X: char = 'J';
const MIN_Y: u8 = 1;
const MAX_Y: u8 = 10;
const X_OFFSET: u32 = 40;
const Y_OFFSET: u32 = 12;
const SCALING: u32 = 53;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MensaPosition {
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) expires: DateTime<Utc>,
}

#[poise::command(slash_command, prefix_command, subcommands("add", "plan"))]
pub(crate) async fn cruisine(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

fn parse_cruisine_letters(position: &str) -> Result<(char, u8), Error> {
    if position.len() < 2 || position.len() > 3 {
        return Err(anyhow!("Bad position format, 2-3 characters").into());
    }

    let position = position.to_ascii_uppercase();
    let mut chars : Vec<char> = position.chars().collect();

    let letter = if chars[0].is_ascii_alphabetic() {
        chars.remove(0)
    } else if chars.last().unwrap().is_ascii_alphabetic() {
        chars.pop().unwrap()
    } else {
        return Err(anyhow!("Bad position format, no letter").into());
    };

    let number = str::parse::<u8>(&chars.into_iter().collect::<String>())?;

    if letter < MIN_X
        || letter > MAX_X
        || number < MIN_Y || number > MAX_Y
    {
        return Err(anyhow!(
            "Bad position format, out of bounds: {MIN_X}-{MAX_X}, {MIN_Y}-{MAX_Y}"
        )
        .into());
    }

    Ok((letter, number))
}

#[cfg(test)]
mod tests {
    use super::parse_cruisine_letters;

    #[test]
    fn test_parse_cruisine_letters() {
        for x in 'A'..='J' {
            for y in 1..=10 {
                let pos = format!("{}{}", x, y);
                assert_eq!(parse_cruisine_letters(&pos).unwrap(), (x, y));

                let pos_reverse = format!("{}{}", y, x);
                assert_eq!(parse_cruisine_letters(&pos_reverse).unwrap(), (x, y));
            }
        }
    }

    #[test]
    fn test_reject_invalid_cruisine_letters() {
        assert!(parse_cruisine_letters("A11").is_err());
        assert!(parse_cruisine_letters("K5").is_err());
        assert!(parse_cruisine_letters("A").is_err());
        assert!(parse_cruisine_letters("10").is_err());
    }
}

/// mark your position in the mensa
/// or play battleship
#[poise::command(slash_command, prefix_command)]
pub(crate) async fn add(
    ctx: Context<'_>,
    #[description = "Letter Number (without space)"] position: String,
    #[description = "Time until your position disappears. Use 0 to delete your marker, Default 1 hour"]
    expires: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let (letter, number) = parse_cruisine_letters(&position)?;

    let duration = match expires {
        Some(x) => Duration::from_std(parse_duration::parse(x.as_str())?)?,
        None => DISAPPEAR_TIME,
    };
    if duration.is_zero() {
        ctx.defer_ephemeral().await?;
        ctx.say("Your location was rapidly approached (position was deleted).")
            .await?;
        ctx.data()
            .mensa_state
            .write()
            .unwrap()
            .remove(&ctx.author().id);
        ctx.data()
            .avatar_cache
            .write()
            .unwrap()
            .remove(&ctx.author().id);
        return Ok(());
    }

    let pos = MensaPosition {
        x: letter as u8 - MIN_X as u8,
        y: number - MIN_Y,
        expires: Utc::now().add(duration),
    };
    {
        let mut m = ctx.data().mensa_state.write().unwrap();
        m.insert(ctx.author().id, pos);
    }
    show_plan(ctx).await
}

/// see the plan
#[poise::command(slash_command, prefix_command)]
pub(crate) async fn plan(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    show_plan(ctx).await
}

async fn show_plan(ctx: Context<'_>) -> Result<(), Error> {
    MENSA_PLAN_IMAGE.get_or_init(|| {
        info!("Loading mensa plan image");
        image::open(MENSA_PLAN_PATH).expect("Failed to load mensa plan image")
    });

    {
        let mut mensa_people = ctx.data().mensa_state.write().unwrap();
        mensa_people.retain(|_, pos| Utc::now() <= pos.expires);
    }

    let mut avatars = HashMap::new();
    {
        let mensa_people = ctx.data().mensa_state.read().unwrap();

        for (id, pos) in mensa_people.iter() {
            avatars
                .entry((pos.x, pos.y))
                .or_insert(HashSet::new())
                .insert(id.clone());
        }
    }

    let mut image = MENSA_PLAN_IMAGE
        .get()
        .context("MENSA_PLAN_IMAGE loaded")?
        .clone();

    for (pos, users) in avatars {
        let imgs = try_join_all(users.iter().map(|x| get(ctx, x))).await?;

        let stitch = Stitch::builder()
            .images(imgs)
            .height_limit(SCALING)
            .width_limit(SCALING)
            .stitch()?;
        let x = X_OFFSET + pos.0 as u32 * SCALING;
        let y = Y_OFFSET + pos.1 as u32 * SCALING;
        image.copy_from(&stitch, x, y)?;
    }

    debug!("Sending updated mensa plan");
    send_image(ctx, image, "mensa_plan.png".to_string()).await
}

async fn get(ctx: Context<'_>, id: &UserId) -> Result<DynamicImage, Error> {
    let user = &id.to_user(ctx).await?;
    load_avatar(&ctx, &user).await
}
