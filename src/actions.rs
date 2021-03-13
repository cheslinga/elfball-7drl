use crate::prelude::*;

impl Object {
    pub fn try_move(&mut self, dest: Point, map: &Map) {
        if self.pos.is_some() {
            if !map.walkable(dest) {
                return
            }
            else {
                self.pos = Some(dest);
                if let Object { viewshed: Some(view), .. } = self {
                    view.refresh = true
                }
            }
        }
        else {
            console::log("ERROR: Entity attempted to move without positional component.")
        }
    }

    pub fn try_attack(&mut self, target: &mut Object, target_id: usize, rng: &mut RandomNumberGenerator, logs: &mut LogBuffer) {
        for member in self.members.iter_mut() {
            if member.attack.is_able() {
                let mut attack_target = 0;
                let mut threat_num = 0;

                //TODO: Add more party member behaviours which will determine who their priority targets are?
                for (i, other_member) in target.members.iter().enumerate() {
                    if other_member.threat.get_threat() > threat_num {
                        threat_num = other_member.threat.get_threat();
                        attack_target = i;
                    }
                }

                let damage = member.attack.roll_for_damage(rng);
                member.threat.add_threat(damage as u32);
                self.inc_attacks.push(TargetedAttack::new((target_id, attack_target), damage));

                logs.update_logs(LogMessage::new()
                    .add_part(format!("{}", member.name), ColorPair::new(member.icon.get_render().1.fg, GREY10))
                    .add_part("attacks", ColorPair::new(WHITE, GREY10))
                    .add_part(format!("{}", target.members[attack_target].name), ColorPair::new(target.members[attack_target].icon.get_render().1.fg, GREY10))
                    .add_part(format!("for {} damage.", damage), ColorPair::new(WHITE, GREY10))
                );
            }
            else {
                member.attack.enable_attack();
            }
        }
    }
}


//Abilities
#[derive(Clone, Copy, PartialEq)]
pub enum Ability {
    Taunt, CureWounds, LesserCureWounds, RallyingCry, KillShot, Deforest, Block, MagicMissile, LesserMagicMissile, PsyBolt, Cleave
}
pub struct StoredAbility {
    pub ability: Ability,
    pub name: String,
    pub on_cooldown: bool,
    pub source_obj: usize,
    pub source_member: usize,
    pub source_ability_id: usize
}
impl StoredAbility {
    pub fn new(ability: Ability, source_obj: usize, source_member: usize, source_ability_id: usize, on_cooldown: bool) -> StoredAbility {
        let name = get_ability_name(ability);
        StoredAbility {
            ability, name, on_cooldown, source_obj, source_member, source_ability_id
        }
    }
    pub fn is_on_cooldown(&self) -> bool { self.on_cooldown }
    pub fn set_source_on_cooldown(&mut self, objects: &mut Vec<Object>) {
        let ability_source = &mut objects[self.source_obj].members[self.source_member].abilities[self.source_ability_id];
        ability_source.set_on_cooldown();
    }
}

pub fn get_ability_name(ability: Ability) -> String {
    match ability {
        Ability::Taunt => String::from("Taunt"),
        Ability::Block => String::from("Block"),
        Ability::CureWounds => String::from("Cure Wounds"),
        Ability::LesserCureWounds => String::from("Lesser Cure"),
        Ability::RallyingCry => String::from("Rallying Cry"),
        Ability::KillShot => String::from("Kill Shot"),
        Ability::Deforest => String::from("Deforest"),
        Ability::MagicMissile => String::from("Magic Missile"),
        Ability::LesserMagicMissile => String::from("Lesser Missile"),
        Ability::Cleave => String::from("Cleave"),
        Ability::PsyBolt => String::from("Psy-Bolt")
    }
}

pub fn get_ability_cooldown(ability: Ability) -> i32 {
    match ability {
        Ability::Taunt => 15,
        Ability::Block => 10,
        Ability::CureWounds => 20,
        Ability::LesserCureWounds => 15,
        Ability::RallyingCry => 20,
        Ability::KillShot => 30,
        Ability::Deforest => 15,
        Ability::MagicMissile => 10,
        Ability::LesserMagicMissile => 20,
        Ability::Cleave => 15,
        Ability::PsyBolt => 3
    }
}

pub fn get_ability_description(ability: Ability) -> String {
    match ability {
        Ability::Taunt => String::from("Increases the amount of threat generated by this Elf for 5 turns."),
        Ability::CureWounds => String::from("Heals the most injured party member for 3d4 hit points."),
        Ability::LesserCureWounds => String::from("Heals the most injured party member for 2d3 hit points."),
        Ability::RallyingCry => String::from("Gives all elves in the party +1 damage to their attacks for 5 turns."),
        Ability::KillShot => String::from("A ranged shot that targets the most injured member of the target party, dealing 2d6 damage."),
        Ability::Deforest => String::from("Chops down all trees directly adjacent to the party."),
        Ability::Block => String::from("Blocks 5 damage from enemy attacks for the next 2 turns."),
        Ability::MagicMissile | Ability::LesserMagicMissile => String::from("An arcane projectile that strikes a random member of a target for 1d3 damage."),
        Ability::Cleave => String::from("Attacks each member of a target within melee range."),
        _ => String::from("")
    }
}

pub fn handle_abilities(objects: &mut Vec<Object>, map: &mut Map, ability: &mut StoredAbility, rng: &mut RandomNumberGenerator, logs: &mut LogBuffer, target: Option<usize>) {
    if !ability.is_on_cooldown() {
        let success = match ability.ability {
            Ability::Taunt => run_taunt(&mut objects[ability.source_obj].members[ability.source_member], logs),
            Ability::Block => run_block(&mut objects[ability.source_obj].members[ability.source_member], logs),
            Ability::CureWounds => run_cure_wounds(&mut objects[ability.source_obj].members, ability.source_member, rng, logs, false),
            Ability::LesserCureWounds => run_cure_wounds(&mut objects[ability.source_obj].members, ability.source_member, rng, logs, true),
            Ability::RallyingCry => run_rallying_cry(&mut objects[ability.source_obj].members, ability.source_member, logs),
            Ability::KillShot => run_killshot(objects, target, (ability.source_obj, ability.source_member), logs, rng),
            Ability::Deforest => run_deforest(objects[ability.source_obj].pos.as_ref().unwrap(), map),
            Ability::MagicMissile | Ability::LesserMagicMissile => run_magic_missile(objects, target, (ability.source_obj, ability.source_member), logs, rng),
            Ability::Cleave => run_cleave(objects, ability.source_obj, ability.source_member, target, logs, rng),
            _ => false
        };
        if success { ability.set_source_on_cooldown(objects); }
    }
    else {
        logs.update_logs(LogMessage::new()
            .add_part(format!("{}'s", objects[ability.source_obj].members[ability.source_member].name), ColorPair::new(objects[ability.source_obj].members[ability.source_member].icon.get_render().1.fg, GREY10))
            .add_part(format!("{} is still on cooldown!", ability.name), ColorPair::new(WHITE, GREY10))
        );
    }
    if let Some(view) = &mut objects[ability.source_obj].viewshed { view.refresh = true; }
}

fn run_taunt(member: &mut PartyMember, logs: &mut LogBuffer) -> bool {
    member.modifiers.push(Modifier::new(ModifierEffect::PlusThreat(5), 5, false));
    member.threat.add_threat(15);
    logs.update_logs(LogMessage::new()
        .add_part(format!("{}",member.name), ColorPair::new(member.icon.get_render().1.fg, GREY10))
        .add_part("lets out a threatening shout, taunting enemies to attack them!", ColorPair::new(WHITE, GREY10))
    );
    return true
}
fn run_block(member: &mut PartyMember, logs: &mut LogBuffer) -> bool {
    member.modifiers.push(Modifier::new(ModifierEffect::Block(5), 2, false));
    logs.update_logs(LogMessage::new()
        .add_part(format!("{}",member.name), ColorPair::new(member.icon.get_render().1.fg, GREY10))
        .add_part("raises their shield, blocking the enemies' blows!", ColorPair::new(WHITE, GREY10))
    );
    member.attack.disable_attack();
    return true
}
fn run_rallying_cry(members: &mut Vec<PartyMember>, caster: usize, logs: &mut LogBuffer) -> bool {
    for member in members.iter_mut() {
        member.modifiers.push(Modifier::new(ModifierEffect::PlusAttack(1), 5, false));
    }

    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", members[caster].name), ColorPair::new(members[caster].icon.get_render().1.fg, GREY10))
        .add_part("lets out a rallying cry, bolstering the party's morale!", ColorPair::new(WHITE, GREY10))
    );
    members[caster].threat.add_threat(5);

    return true
}
fn run_cleave(objects: &mut Vec<Object>, source_obj: usize, source_member: usize, target_obj: Option<usize>, logs: &mut LogBuffer, rng: &mut RandomNumberGenerator) -> bool {
    if target_obj.is_none() {
        logs.update_logs(LogMessage::new()
            .add_part("That ability needs a target!", ColorPair::new(WHITE, GREY10))
        );
        return false
    }
    {
        let obj = &mut objects[target.unwrap()];

        if obj.tag != ActorTag::Enemy || obj.members.is_empty() {
            logs.update_logs(LogMessage::new()
                .add_part("Why would you want to do that?", ColorPair::new(WHITE, GREY10))
            );
            return false
        }
    }

    let target_position = objects[target_obj.unwrap()].pos.as_ref().unwrap().clone();
    let adjacencies = objects[source_obj].pos.as_ref().unwrap().clone().get_neighbors();
    if !adjacencies.contains(&target_position) {
        logs.update_logs(LogMessage::new()
            .add_part("Your target must be in melee range to use this ability!", ColorPair::new(WHITE, GREY10))
        );
        return false
    }

    let idxlist = {
        let enemy_members = &mut objects[target_obj.unwrap()].members;
        let mut vec = Vec::new();
        for (i, _) in enemy_members.iter().enumerate() { vec.push(i); }
        vec
    };

    let player = &mut objects[source_obj];
    let amt = player.members[source_member].attack.roll_for_damage(rng);
    for idx in idxlist.iter() {
        player.inc_attacks.push(TargetedAttack::new((target_obj.unwrap(), *idx), amt));
    }
    player.members[source_member].attack.disable_attack();
    player.members[source_member].threat.add_threat(15 * idxlist.len() as u32);

    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", player.members[source_member].name), ColorPair::new(player.members[source_member].icon.get_render().1.fg, GREY10))
        .add_part("attacks each unit in", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", objects[target_obj.unwrap()].name.clone()), ColorPair::new(objects[target_obj.unwrap()].render.as_ref().unwrap().get_render().1.fg.clone(), GREY10))
        .add_part("for", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", amt), ColorPair::new(GOLD, GREY10))
        .add_part("damage.", ColorPair::new(WHITE, GREY10))
    );

    return true
}

fn run_cure_wounds(members: &mut Vec<PartyMember>, caster_id: usize, rng: &mut RandomNumberGenerator, logs: &mut LogBuffer, lesser: bool) -> bool {
    let health_list = {
        let mut vec = members.iter().enumerate()
            .map(|(i, m)| (i, m.health.get_max() - m.health.get_life()))
            .collect::<Vec<(usize, i32)>>();
        vec.sort_by(|a,b| b.1.cmp(&a.1));
        vec
    };
    let amt = if !lesser { rng.roll_dice(3,4) }
        else{ rng.roll_dice(2, 3) };
    members[health_list[0].0].health.gain_life(amt);

    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", members[caster_id].name.clone()), ColorPair::new(members[caster_id].icon.get_render().1.fg, GREY10))
        .add_part("casts a healing wave upon", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", members[health_list[0].0].name), ColorPair::new(members[health_list[0].0].icon.get_render().1.fg, GREY10))
        .add_part("for", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", amt), ColorPair::new(GOLD, GREY10))
        .add_part("HP.", ColorPair::new(WHITE, GREY10))
    );

    members[caster_id].attack.disable_attack();

    return true
}

fn run_deforest(source_pos: &Point, map: &mut Map) -> bool {
    let neighbor_list = {
        let mut vec = Vec::new();
        //let mut second_vec = Vec::new();

        vec.append(&mut source_pos.get_neighbors());
        /*
        for p in vec.iter() {
            second_vec.append(&mut p.get_neighbors());
        }
        vec.append(&mut second_vec);
        vec.dedup();
         */
        vec.retain(|p| p.x >= 1 && p.y >= 1 && p.x <= map.width - 2 && p.y <= map.height - 2);

        vec
    };

    let idx_list = neighbor_list.iter()
        .map(|i| map.point2d_to_index(*i))
        .collect::<Vec<usize>>();

    let mut treecount = 0;
    for idx in idx_list.into_iter() {
        if map.tiles[idx] == TileClass::Tree {
            map.tiles[idx] = TileClass::ForestFloor;
            treecount += 1;
        }
    }

    return if treecount > 0 { true } else { false }
}

fn run_killshot(objects: &mut Vec<Object>, target: Option<usize>, source_ids: (usize, usize), logs: &mut LogBuffer, rng: &mut RandomNumberGenerator) -> bool {
    if target.is_none() {
        logs.update_logs(LogMessage::new()
            .add_part("That ability needs a target!", ColorPair::new(WHITE, GREY10))
        );
        return false
    }

    let amt = rng.roll_dice(2, 6);
    let mut idx = 0;
    {
        let obj = &mut objects[target.unwrap()];

        if obj.tag != ActorTag::Enemy || obj.members.is_empty() {
            logs.update_logs(LogMessage::new()
                .add_part("Why would you want to do that?", ColorPair::new(WHITE, GREY10))
            );
            return false
        }

        let health_list = {
            let mut vec = obj.members.iter().enumerate()
                .map(|(i, m)| (i, m.health.get_max() - m.health.get_life()))
                .collect::<Vec<(usize, i32)>>();
            vec.sort_by(|a, b| b.1.cmp(&a.1));
            vec
        };
        idx = health_list[0].0;
    }
    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", objects[source_ids.0].members[source_ids.1].name), ColorPair::new(objects[source_ids.0].members[source_ids.1].icon.get_render().1.fg, GREY10))
        .add_part("fires a deadly shot at", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", objects[target.unwrap()].members[idx].name), ColorPair::new(objects[target.unwrap()].members[idx].icon.get_render().1.fg, GREY10))
        .add_part("for", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", amt), ColorPair::new(GOLD, GREY10))
        .add_part("damage.", ColorPair::new(WHITE, GREY10))
    );

    objects[source_ids.0].members[source_ids.1].attack.disable_attack();
    objects[source_ids.0].members[source_ids.1].threat.add_threat(30);
    objects[source_ids.0].inc_attacks.push(TargetedAttack::new((target.unwrap(), idx), amt));

    return true
}

fn run_magic_missile(objects: &mut Vec<Object>, target: Option<usize>, source_ids: (usize, usize), logs: &mut LogBuffer, rng: &mut RandomNumberGenerator) -> bool {
    if target.is_none() {
        logs.update_logs(LogMessage::new()
            .add_part("That ability needs a target!", ColorPair::new(WHITE, GREY10))
        );
        return false
    }
    {
        let obj = &mut objects[target.unwrap()];

        if obj.tag != ActorTag::Enemy || obj.members.is_empty() {
            logs.update_logs(LogMessage::new()
                .add_part("Why would you want to do that?", ColorPair::new(WHITE, GREY10))
            );
            return false
        }
    }

    let idx = rng.range(0,objects[target.unwrap()].members.len());
    let amt = rng.roll_dice(1, 3);

    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", objects[source_ids.0].members[source_ids.1].name), ColorPair::new(objects[source_ids.0].members[source_ids.1].icon.get_render().1.fg, GREY10))
        .add_part("casts an arcane missile toward", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", objects[target.unwrap()].members[idx].name), ColorPair::new(objects[target.unwrap()].members[idx].icon.get_render().1.fg, GREY10))
        .add_part("for", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", amt), ColorPair::new(GOLD, GREY10))
        .add_part("damage.", ColorPair::new(WHITE, GREY10))
    );

    objects[source_ids.0].members[source_ids.1].attack.disable_attack();
    objects[source_ids.0].members[source_ids.1].threat.add_threat(5);
    objects[source_ids.0].inc_attacks.push(TargetedAttack::new((target.unwrap(), idx), amt));

    return true
}

pub fn run_psybolt(objects: &mut Vec<Object>, target: Option<usize>, source_ids: (usize, usize), logs: &mut LogBuffer, rng: &mut RandomNumberGenerator) -> bool {
    if target.is_none() {
        return false
    }
    {
        let obj = &mut objects[target.unwrap()];

        if obj.tag != ActorTag::Player || obj.members.is_empty() {
            return false
        }
    }

    let idx = rng.range(0,objects[target.unwrap()].members.len());
    let amt = rng.roll_dice(1, 3);

    logs.update_logs(LogMessage::new()
        .add_part(format!("{}", objects[source_ids.0].members[source_ids.1].name), ColorPair::new(objects[source_ids.0].members[source_ids.1].icon.get_render().1.fg, GREY10))
        .add_part("casts a psychic bolt toward", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", objects[target.unwrap()].members[idx].name), ColorPair::new(objects[target.unwrap()].members[idx].icon.get_render().1.fg, GREY10))
        .add_part("for", ColorPair::new(WHITE, GREY10))
        .add_part(format!("{}", amt), ColorPair::new(GOLD, GREY10))
        .add_part("damage.", ColorPair::new(WHITE, GREY10))
    );

    objects[source_ids.0].members[source_ids.1].attack.disable_attack();
    objects[source_ids.0].members[source_ids.1].threat.add_threat(5);
    objects[source_ids.0].inc_attacks.push(TargetedAttack::new((target.unwrap(), idx), amt));

    return true
}
