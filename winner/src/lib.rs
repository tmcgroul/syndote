#![no_std]
use gstd::{exec, msg, prelude::*, ActorId};
use syndote_io::*;

pub const COST_FOR_UPGRADE: u32 = 500;
pub const FINE: u32 = 1_000;

#[gstd::async_main]
async fn main() {
    let monopoly_id = msg::source();
    let message: YourTurn = msg::load().expect("Unable to decode struct`YourTurn`");
    let (_, mut my_player) = message
        .players
        .iter()
        .find(|(player, _player_info)| player == &exec::program_id())
        .expect("Can't find my address")
        .clone();

    if my_player.in_jail {
        let reply: GameEvent = msg::send_for_reply_as(
            monopoly_id,
            GameAction::ThrowRoll {
                pay_fine: false,
                properties_for_sale: None,
            },
            0,
        )
        .expect("Error in sending a message `GameAction::ThrowRoll`")
        .await
        .expect("Unable to decode `GameEvent");

        if let GameEvent::Jail { in_jail, position } = reply {
            if !in_jail {
                my_player.position = position;
            } else {
                msg::reply("", 0).expect("Error in sending a reply to monopoly contract");
                return;
            }
        }
    }

    let position = my_player.position;

    let (my_cell, free_cell, gears, price) =
        if let Some((account, gears, price, _)) = &message.properties[position as usize] {
            let my_cell = account == &exec::program_id();
            let free_cell = account == &ActorId::zero();
            (my_cell, free_cell, gears, price)
        } else {
            msg::reply("", 0).expect("Error in sending a reply to monopoly contract");
            return;
        };

    if my_cell {
        if gears.len() < 3 && my_player.balance > COST_FOR_UPGRADE + FINE {
            msg::send_for_reply_as::<_, GameEvent>(
                monopoly_id,
                GameAction::AddGear {
                    properties_for_sale: None,
                },
                0,
            )
            .expect("Error in sending a message `GameAction::AddGear`")
            .await
            .expect("Unable to decode `GameEvent");
            msg::reply("", 0).expect("Error in sending a reply to monopoly contract");
            return;
        }
    }

    if free_cell {
        if my_player.balance > price + FINE {
            msg::send_for_reply_as::<_, GameEvent>(
                monopoly_id,
                GameAction::BuyCell {
                    properties_for_sale: None,
                },
                0,
            )
            .expect("Error in sending a message `GameAction::BuyCell`")
            .await
            .expect("Unable to decode `GameEvent");
        }
    } else if !my_cell {
        msg::send_for_reply_as::<_, GameEvent>(
            monopoly_id,
            GameAction::PayRent {
                properties_for_sale: None,
            },
            0,
        )
        .expect("Error in sending a message `GameAction::PayRent`")
        .await
        .expect("Unable to decode `GameEvent");
    }
    msg::reply("", 0).expect("Error in sending a reply to monopoly contract");
}
