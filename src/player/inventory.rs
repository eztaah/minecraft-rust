use crate::constants::MAX_ITEM_SLOTS;
use crate::constants::MAX_ITEM_STACK;
use crate::items;
use crate::items::Item;
use crate::player::Player;
use bevy::prelude::*;

// Ajoute un item à l'inventaire du joueur
pub fn add_item_to_inventory(
    player: &mut Query<&mut Player>,
    item_id: items::ItemsType,
    mut nb: u32,
) {
    let mut player = player.single_mut();

    for i in 0..MAX_ITEM_SLOTS {
        let item_option = player.inventory.get(&i);

        if item_option.is_some() {
            let existing_item = item_option.expect("Error : empty item");
            // If not item of right type or stack already full : pass
            if existing_item.id != item_id || existing_item.nb >= MAX_ITEM_STACK {
                continue;
            }

            nb += existing_item.nb;
        }

        let inserted_items = if nb >= MAX_ITEM_STACK {
            MAX_ITEM_STACK
        } else {
            nb
        };
        nb -= inserted_items;

        // Push inserted items in right inventory slot
        player.inventory.insert(
            i,
            items::Item {
                id: item_id,
                nb: inserted_items,
            },
        );

        // If no more items to add, end loop
        if nb == 0 {
            break;
        }
    }

    // Problem : if inventory full, items disappear
}

// Retire un item de l'inventaire du joueur
pub fn remove_item_from_inventory(
    player: &mut Query<&mut Player>,
    item_id: items::ItemsType,
    mut nb: u32,
) {
    let mut player = player.single_mut();
    for i in 0..MAX_ITEM_SLOTS {
        let item_option = player.inventory.get(&i);

        if item_option.is_none() {
            continue;
        }

        let existing_stack = *item_option.expect("Error : empty item");

        if existing_stack.id != item_id {
            continue;
        }

        if existing_stack.nb - nb == 0 {
            player.inventory.remove(&i);
            nb -= existing_stack.nb;
        } else {
            // Push inserted items in right inventory slot
            player.inventory.insert(
                i,
                items::Item {
                    id: item_id,
                    nb: existing_stack.nb - nb,
                },
            );
            nb = 0;
        }

        // If no more items to remove, end loop
        if nb == 0 {
            break;
        }
    }
}

/// Add items to stack at specified position\
/// Stacks cannot exceed MAX_ITEM_STACK number of items\
/// Returns number of items really added to the stack
pub fn add_item_to_stack(
    player: &mut Player,
    item_id: items::ItemsType,
    stack: u32,
    mut nb: u32,
) -> u32 {
    let item_option = player.inventory.get(&stack);
    let mut new_item = Item { id: item_id, nb };

    if let Some(item) = item_option {
        if nb + item.nb > MAX_ITEM_STACK {
            nb = MAX_ITEM_STACK - item.nb;
        }
        new_item.nb = nb + item.nb;
    }
    player.inventory.insert(stack, new_item);
    nb
}

/// Removes items from stack at specified position\
/// Stacks cannot have < 0 number of items\
/// Returns number of items really removed from the stack
pub fn remove_item_from_stack(
    player: &mut Player,
    item_id: items::ItemsType,
    stack: u32,
    mut nb: u32,
) -> u32 {
    let item_option = player.inventory.get(&stack);

    if let Some(item) = item_option {
        let item_nb = item.nb;
        if nb > item_nb {
            nb = item_nb;
            player.inventory.remove(&stack);
        } else {
            player.inventory.insert(
                stack,
                Item {
                    id: item_id,
                    nb: item_nb - nb,
                },
            );
        }
        return nb;
    }
    0
}

// Retourne le nombre d'items dans l'inventaire du joueur
// pub fn get_item_count(player: &Player, item_id: i32) -> i32 {
//     for item in player.inventory.iter() {
//         if item.id == item_id {
//             return item.nb;
//         }
//     }
//     return 0;
// }

// Retourne true si le joueur possède l'item
pub fn has_item(player: &mut Query<&mut Player>, item_id: items::ItemsType) -> bool {
    let player = player.single_mut();
    for item in player.inventory.values() {
        if item.id == item_id {
            return true;
        }
    }
    false
}