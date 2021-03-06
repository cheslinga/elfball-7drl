use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct TargetedAttack {
    pub target: (usize, usize),
    pub damage: i32
}
impl TargetedAttack {
    pub fn new(target: (usize, usize), damage: i32) -> TargetedAttack { TargetedAttack { target, damage } }
}

pub fn process_combat(objects: &mut Vec<Object>, logs: &mut LogBuffer, player_death: &mut bool, player_targets: &mut TargetList, map: &Map, fkills: &mut u32, bkills: &mut u32) {
    let mut attack_list: Vec<(usize, TargetedAttack)> = Vec::new();
    let mut kill_list: Vec<(usize, usize)> = Vec::new();
    let mut check_ai_list: Vec<(usize, usize)> = Vec::new();
    let mut refresh_targets = false;

    //Save the targeted attack and the ID of the object triggering it
    for (i, obj) in objects.iter_mut().enumerate() {
        //Also determine if the parties should be gaining threat or resetting
        let should_gain_threat = obj.in_combat;
        for member in obj.members.iter_mut() {
            if should_gain_threat {
                member.threat.increment_threat();
            } else {
                member.threat.reset_threat();
            }
        }

        if !obj.inc_attacks.is_empty() {
            for a in obj.inc_attacks.iter() {
                attack_list.push((i, a.clone()));
                check_ai_list.push((i, a.target.0));
            }
        }
    }

    for c in check_ai_list.iter() {
        let player_pos = &objects[0].pos.as_ref().unwrap().clone();
        if let Some(ai) = &mut objects[c.1].ai {
            if c.0 == 0 {
                ai.target = Some(0);
                ai.state = AIState::Chasing;
                ai.tgt_memory = 24;
                ai.tgt_heatmap.reset_to_single_node(player_pos);
            }
        }
    }

    //Clear the incoming attacks
    for a in attack_list.iter() {
        objects[a.0].inc_attacks.clear();
    }
    //Process the damage against the targeted party member's health
    for a in attack_list.iter() {
        let target = &mut objects[a.1.target.0].members[a.1.target.1];
        target.health.lose_life(a.1.damage);

        if target.health.get_life() <= 0 {
            kill_list.push((a.1.target.0, a.1.target.1));
        }
    }
    //Kill anything that was added to the kill list
    kill_list.dedup();
    for k in kill_list.iter() {
        let object_party = &mut objects[k.0].members;

        logs.update_logs(LogMessage::new()
            .add_part(format!("{}", object_party[k.1].name), ColorPair::new(object_party[k.1].icon.get_render().1.fg, GREY10))
            .add_part("has been slain.", ColorPair::new(WHITE, GREY10))
        );

        if object_party[k.1].class == "Beast" {
            *bkills += 1
        }
        else {
            *fkills += 1
        }

        object_party.remove(k.1);
        object_party.shrink_to_fit();

        //Remove the whole object if the party is empty (but also not the player)
        if object_party.is_empty() {
            if objects[k.0].tag != ActorTag::Player {
                refresh_targets = true;
                logs.update_logs(LogMessage::new()
                    .add_part("You have defeated the", ColorPair::new(WHITE, GREY10))
                    .add_part(format!("{}.", objects[k.0].name), ColorPair::new(objects[k.0].render.as_ref().unwrap().get_render().1.fg, GREY10))
                );
                objects.remove(k.0);
            } else {
                *player_death = true;
            }
        }
    }

    //Mark off the player as no longer in combat if there are no enemies around them
    let player_view = objects[0].viewshed.as_ref().unwrap().visible.to_vec();
    let mut still_in_combat = {
        let mut result = false;
        for obj in objects.iter() {
            if let Object { pos: Some(pos), tag, .. } = obj {
                if player_view.contains(pos) && tag == &ActorTag::Enemy {
                    result = true
                }
            }
        }
        result
    };
    objects[0].in_combat = still_in_combat;

    if refresh_targets { player_targets.reset_targets(objects, map) }
}

pub fn reset_attack_capabilities(members: &mut Vec<PartyMember>) {
    for member in members.iter_mut() {
        member.attack.enable_attack();
    }
}

pub fn process_all_cooldowns(objects: &mut Vec<Object>) {
    for obj in objects.iter_mut() {
        for member in obj.members.iter_mut() {
            for ability in member.abilities.iter_mut() {
                if ability.is_on_cooldown() { ability.increment_cd_timer(); }
            }
        }
    }
}