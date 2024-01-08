#![warn(missing_docs)]

//! A Bevy plugin that allows the developer to interact with the Discord Presence API with ease
//!
//! This plugin is a Bevy wrapper around the [Discord Presence](https://docs.rs/crate/discord-presence) crate which in turn is a wrapper around the [Discord Presence API](https://discordapp.com/developers/docs/game-sdk/discord-presence).
//! # Examples
//!
//! ```rust no_run
//! use bevy::prelude::*;
//! use bevy_discord_presence::{ActivityState, RPCConfig, RPCPlugin};
//!
//! fn main() {
//!     println!("hello world!");
//!     let mut app = App::new();
//!     app.add_plugins(( 
//!         DefaultPlugins, 
//!         RPCPlugin {
//!             config: RPCConfig {
//!                 app_id: 425407036495495169,
//!                 show_time: true,
//!             }
//!         }
//!     ));
//!     app.add_systems(Update, update_presence);
//!
//!     app.run();
//! }
//!
//! fn update_presence(mut state: ResMut<ActivityState>) {
//!     state.details = Some("Hello World".to_string());
//! }
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{log::prelude::*, prelude::*};
use discord_presence::{models::ActivityTimestamps, Client as DiscordClient, Event};

/// The Discord configuration
pub mod config;
/// The state that holds the Discord activity
mod state;

/// A wrapper around the internal [`discord_presence::Client`] struct that implements [`bevy::prelude::Resource`]
#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
pub struct Client(DiscordClient);

impl Client {
    /// Instantiates a [`Client`] struct
    ///
    /// Wraps the internal [`discord_presence::Client`] struct
    pub fn new(client_id: u64) -> Self {
        Client(DiscordClient::new(client_id))
    }
}

pub use config::{RPCConfig, RPCPlugin};
pub use state::ActivityState;

/// Implements the Bevy plugin trait
impl Plugin for RPCPlugin {
    fn build(&self, app: &mut App) {
        let client_config = self.config;

        // NOTE: I am aware this is deprecated
        // For now, for the sake of backwards compatability with old Bevy versions we will keep using this
        // If Bevy removes these functions in future, this will change
        app.add_systems(Startup, startup_client);
        app.add_systems(Update, check_activity_changed);
        debug!("Added systems");

        app.insert_resource::<RPCConfig>(client_config);

        app.init_resource::<ActivityState>();
        app.insert_resource::<Client>(Client::new(client_config.app_id));

        debug!("Initialized resources");
    }

    fn name(&self) -> &str {
        "Discord Presence"
    }
}

/// Initializes the client and starts it running
fn startup_client(
    mut activity: ResMut<ActivityState>,
    mut client: ResMut<Client>,
    config: Res<RPCConfig>,
) {
    use quork::traits::list::ListVariants;

    if config.show_time {
        activity.timestamps = Some(ActivityTimestamps {
            start: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time has gone backwards")
                    .as_secs(),
            ),
            end: None,
        });
    }

    for event in Event::VARIANTS {
        client.on_event(event, {
            let events = activity.events.clone();

            move |_| {
                events.lock().0.push_back(event);
                debug!("Added event: {:?}", event);
            }
        });
    }

    _ = client.start();
    debug!("Client has started");
}

/// Runs whenever the activity has been changed, and at startup
fn check_activity_changed(activity: Res<ActivityState>, mut client: ResMut<Client>) {
    if activity.is_changed() {
        let res = client.set_activity(|_| activity.clone().into());

        if let Err(why) = res {
            error!("Failed to set presence: {}", why);
        }
    }
}
