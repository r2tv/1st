/*
    Appellation: telegram <module>
    Contrib: FL03 <j3mccain@gmail.com> (https://github.com/FL03)
    Description: ... Summary ...
*/
use super::{TelegramBotSpec, DEFAULT_ENV_KEY};
use crate::services::openai::{clean_choices, OpenAI};

use acme::AsyncSpawable;
use scsys::AsyncResult;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::repls::CommandReplExt;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct TelegramBotConfig {
    pub name: String,
    token: String,
    pub username: String,
}

impl TelegramBotConfig {
    pub fn new(name: String, token: String, username: String) -> Self {
        Self {
            name,
            token,
            username,
        }
    }
    pub fn from_env(token: Option<&str>) -> Self {
        let token = std::env::var(token.unwrap_or(DEFAULT_ENV_KEY))
            .ok()
            .unwrap();
        Self::new(Default::default(), token, Default::default())
    }
    pub fn try_from_env(token: Option<&str>) -> AsyncResult<Self> {
        let token = std::env::var(token.unwrap_or(DEFAULT_ENV_KEY))?;
        Ok(Self::new(Default::default(), token, Default::default()))
    }
}

impl Default for TelegramBotConfig {
    fn default() -> Self {
        Self::from_env(None)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct TelegramBot {
    pub cnf: TelegramBotConfig,
}

impl TelegramBot {
    pub fn new(cnf: TelegramBotConfig) -> Self {
        Self { cnf }
    }
}

impl TelegramBotSpec for TelegramBot {
    fn name(&self) -> String
    where
        Self: Sized,
    {
        self.cnf.name.clone()
    }

    fn username(&self) -> String
    where
        Self: Sized,
    {
        self.cnf.username.clone()
    }

    fn bot(&self) -> Bot
    where
        Self: Sized,
    {
        Bot::new(self.cnf.token.clone())
    }
}

#[async_trait::async_trait]
impl AsyncSpawable for TelegramBot {
    async fn spawn(&mut self) -> AsyncResult<&Self> {
        Command::repl(self.bot(), handler).await;
        Ok(self)
    }
}

/// Defines the desired command structure for the bot
#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Rolls a 6-sided die")]
    Dice,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Given a topic or url, return a concise summary")]
    Query(String),
}

async fn handle_oai_query(bot: &Bot, msg: Message, prompt: String) -> ResponseResult<()> {
    let oai = OpenAI::default();
    let req = oai.create_request(prompt.as_str());
    let res = oai.response(req).await.expect("");
    bot.send_message(msg.chat.id, clean_choices(res)).await?;
    Ok(())
}

/// Handles the commands issued to the bot and returns a [ResponseResult]
async fn handler(bot: Bot, cmd: Command, msg: Message) -> ResponseResult<()> {
    match cmd {
        Command::Dice => {
            bot.send_dice(msg.chat.id).await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Query(prompt) => {
            handle_oai_query(&bot, msg, prompt).await?;
        }
    };

    Ok(())
}
