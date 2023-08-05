use crate::hackaton_bot_state::*;
use crate::msg::{self, send_main_msg, send_main_msg_custom_text};
use async_trait::async_trait;
use log::info;
use log::warn;
use mobot::{handler::BotState, *};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task;

pub struct HandlerWrapper<S: BotState> {
    /// Wraps the async handler function.
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn BotHandler<S>>,

    /// State related to the context in which it is called (e.g., a chat ID or a user ID)
    pub state: State<S>,
}

impl HandlerWrapper<HackatonBotState> {
    /// Create a new `Handler` from a `BotHandlerFn`.
    pub fn new(
        func: impl Into<Box<dyn BotHandler<HackatonBotState>>>,
    ) -> Box<dyn BotHandler<HackatonBotState>> {
        Box::new(Self {
            f: func.into(),
            state: State::default(),
        })
    }

    /// Attach a state to the handler.
    pub fn with_state(self, state: HackatonBotState) -> Self {
        Self {
            f: self.f,
            state: State::new(state),
        }
    }
}

#[async_trait]
impl BotHandler<HackatonBotState> for HandlerWrapper<HackatonBotState> {
    fn get_state(&self) -> &State<HackatonBotState> {
        &self.state
    }

    fn set_state(&mut self, state: Arc<RwLock<HackatonBotState>>) {
        task::block_in_place(|| self.state = State::new(state.blocking_read().clone()));
    }
}

#[async_trait]
impl BotHandlerFn<HackatonBotState> for HandlerWrapper<HackatonBotState> {
    async fn run(
        &self,
        event: Event,
        state: State<HackatonBotState>,
    ) -> Result<Action, anyhow::Error> {
        match self.f.run(event.clone(), state.clone()).await {
            Ok(a) => match a {
                Action::ReplyText(msg) => {
                    warn!("Unexpected failure: {}", msg);
                    Ok(Action::ReplyText(msg))
                }
                _ => Ok(a),
            },
            Err(e) => {
                info!("Wrapper catched err: {}", e);
                Ok(Action::ReplyText(String::from("Ошибка со стороны сервера - попробуйте позже")))
            }
        }
    }
}
