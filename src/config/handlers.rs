use evalexpr::{Node, build_operator_tree, context_map};
use serde::Deserialize;

use crate::config::ConfigDe;

#[derive(Deserialize, Clone, Default)]
pub struct HandlersConfig {
    #[serde(rename = "on-invite")]
    pub on_invite: OnInviteHandlerConfig,
}

impl ConfigDe for HandlersConfig {
    const PATH: &'static str = "handlers";
}

#[derive(Deserialize, Clone, Debug)]
pub struct OnInviteHandlerConfig {
    #[serde(rename = "initial-delay")]
    initial_delay: f64,
    #[serde(rename = "retry-delay")]
    retry_delay: Node,
    #[serde(rename = "retry-condition")]
    retry_condition: Node,
}

impl Default for OnInviteHandlerConfig {
    fn default() -> Self {
        Self {
            initial_delay: 2.0,
            retry_delay: build_operator_tree("prev * 2").unwrap(),
            retry_condition: build_operator_tree("n > 10").unwrap(),
        }
    }
}

impl OnInviteHandlerConfig {
    pub fn initial_delay(&self) -> f64 {
        self.initial_delay
    }

    pub fn delay(&self, prev: f64, n: u32) -> f64 {
        let context = context_map! {
            "prev" => float prev,
            "n" => int n
        }
        .unwrap();

        self.retry_delay
            .eval_with_context(&context)
            .unwrap()
            .as_number()
            .unwrap()
    }

    pub fn should_retry(&self, delay: f64, n: u32) -> bool {
        let context = context_map! {
            "delay" => float delay,
            "n" => int n
        }
        .unwrap();

        self.retry_condition
            .eval_with_context(&context)
            .unwrap()
            .as_boolean()
            .unwrap()
    }
}
