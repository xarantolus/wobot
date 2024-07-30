# WoBot

## Features

WoBot comes with a ton of features, including:

##### Improved Events:

create a Discord Event, a thread and allow RSVP via reactions with a **single** command  
export all events to your calendar (works on mobile)

##### Improved Emojis:

`add`, `rename` and `remove`  
`upload` to convert images into emojis  
`copy` emojis from other servers to your own

##### Easy Reaction roles

simply react with the emojis you want

##### MEMEs:

Obama: when someone congratulates themselves  
Cutie Pie: tell your friends how cute they are

##### ToDo list

store anything you like

#### Reminder

schedule whatever you like for later

#### Mensa plan

know what's up for lunch  
show the next available plan  
automatically skips weekends

## Images

Not yet convinced? Have some images:

![](images/reaction_roles.png)
![](images/emoji.png)
![](images/mensa.png)
![](images/todo.png)
![](images/obama.png)
![](images/cutie_pie.png)

## Contributing

If you have a great idea or suggestion, feel free to open [an issue](https://github.com/Friendly-Banana/wobot/issues).
If you want this feature right now and can code, open [a pull request](https://github.com/Friendly-Banana/wobot/pulls).
Please make sure to run `cargo fmt` before committing.

### Running the Bot

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. [Install Shuttle](https://docs.shuttle.rs/getting-started/installation)
3. Optional: [Install PostgresQL](https://www.postgresql.org/download/), you can also use a Docker container
4. Change the Database URL (`postgres://test:pass@localhost:5432/postgres`) in `main.rs` and `.env` to your local
   PostgresQL instance, leave it blank to use Docker.
5. Execute `setup.sql` in your DB
6. Create a Discord Bot on the [Discord Developer Portal](https://discord.com/developers/applications)
7. Copy the bot token and put it in a `Secrets.toml` file in the root directory:
    ```toml
    DISCORD_TOKEN = "your token here"
    ```
   You can also create a `Secrets.dev.toml` file if you want to test with different tokens in development.
8. Invite the bot to your server with the `ADMINISTRATOR` permission, you can also only choose the permissions you need.
9. Run the bot with `cargo run`

Some features also require a font and images from the `assets` folder.
Due to legal reasons, not all of them can be provided here. What's missing:

- `Rockwill.ttf`: [Rockwill](https://fontmeme.com/fonts/rockwill-font/)
- `obama_medal.jpg`: [Obama Medal](https://a.pinatafarm.com/1015x627/ade80aa63d/obama-medal.jpg)
- `mensa_plan.png`: [Mensa Plan](https://www.meck-architekten.de/projekte/id/2019-mensa-campus-garching/) or
  from [here](https://www.heinze.de/architekturobjekt/zoom/12979688/)

Simply download them and place them in the `assets` folder with the same name.

## Technical Overview

WoBot is a Discord Bot written in [Rust](https://www.rust-lang.org/)
with [the  Poise framework](https://github.com/serenity-rs/poise/).
It's hosted on [Shuttle](https://www.shuttle.rs/) and uses a PostgresQL database.

The mensa plan uses the [Eat API](https://tum-dev.github.io/eat-api), the mensa coordinates link
to [Google Maps](https://www.google.com/maps).

The mensaplan API is written by myself and may become public in the future.

### Configuration

`config.hjson` uses a human-friendly JSON version, [HJson](https://hjson.github.io/).

You can set up automatic reactions and replies based on keywords. All of them are case-insensitive.
Auto-reactions also support regex and match on word boundaries, ignoring punctuation around them.
WoBot can react with both Unicode and custom Discord emojis, even animated ones.

#### Example Config

```hjson
{
  // channel for event threads
  event_channel_per_guild: {
    // guild_id: channel_id
    0: 0
  }
  role_removals: [
    {
      // If someone in guild has not written in channel in timeout_days days, the role gets removed
      guild_id: 1,
      channel_id: 2,
      role_id: 3,
      timeout_days: 30
    }
  ]
  auto_reactions: {
    robot: {
      name: "🤖"
    }
    vibing: {
      animated: true
      name: vibing
      // emoji id
      id: 0
    }
  }
  auto_replies: [
    {
      keywords: [
        "wobot info"
        "wobot help"
      ]
      // discord user id
      user: 0
      title: About WoBot
      description: "Hi, I'm **WoBot**, your friendly neighborhood bot. Please send any questions or feedback to my author, {user}. This message was sent {count} times. Have a nice day!"
      colour: 15844367
    }
}
```
