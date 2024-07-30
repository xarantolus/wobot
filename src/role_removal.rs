use std::time::Duration;


use chrono::Days;
use poise::serenity_prelude::{
    futures::StreamExt, Context, Timestamp,
};
use tokio::time::interval;
use tracing::{error};

use crate::{Error, RoleRemoval};

pub(crate) fn check_role_removals(ctx: Context, removal_rules: Vec<RoleRemoval>, period: Duration) {
    tokio::spawn(async move {
        let mut interval = interval(period);

        loop {
            interval.tick().await;

            if let Err(why) = check_expired_roles(&ctx, &removal_rules).await {
                error!("Failed checking role removal: {}", why);
            }
        }
    });
}

pub(crate) async fn check_expired_roles(
    ctx: &Context,
    removal_rules: &Vec<RoleRemoval>,
) -> Result<(), Error> {
    for rule in removal_rules {
        let now = Timestamp::now();
        let _cutoff_date = now
            .checked_sub_days(Days::new(rule.timeout_days))
            .expect("cutoff date calculation should not fail");

        // List members with role
        let members_with_role = rule
            .guild_id
            .members_iter(ctx)
            .filter_map(|member| async move {
                let member = member.ok()?;
                if member.roles.contains(&rule.role_id) {
                    Some(member)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .await;

        for member in members_with_role {
            // We want to take joined date or latest message, whichever is later
            let _joined_at = member.joined_at.unwrap_or(now);

            unimplemented!("Check if member joined after cutoff date, if so, remove role");
        }
    }

    Ok(())
}
