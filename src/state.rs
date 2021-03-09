use crate::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ContextState {
    MainMenu, InGame, Paused, GameMenu
}
#[derive(Clone, Copy, PartialEq)]
pub enum TurnState {
    Player, AI, GameOver
}

pub struct State {
    proc: bool,
    refresh: bool,
    pub passed_turn: bool,
    pub go_next_level: bool,
    pub status: ContextState,
    pub turn_state: TurnState,
    pub world: World,
    pub logs: LogBuffer
}
impl State {
    pub fn init() -> State {
       let mut logs = LogBuffer::new();
       logs.update_logs(LogMessage::new()
           .add_part("Your ancestors have called upon you to save these lands from the bestial scourge they are beset against.", ColorPair::new(WHITE, GREY10))
           .add_part("Gather more", ColorPair::new(WHITE, GREY10))
           .add_part("Elves (☻)", ColorPair::new(LIME_GREEN, GREY10))
           .add_part("to expand your party of woodland defenders.", ColorPair::new(WHITE, GREY10))
           .add_part("Beware of", ColorPair::new(WHITE, GREY10))
           .add_part("Beasts (b)", ColorPair::new(RED, GREY10))
           .add_part("that stalk these woods, and", ColorPair::new(WHITE, GREY10))
           .add_part("the Forsaken (☺),", ColorPair::new(PURPLE, GREY10))
           .add_part("your fallen bretheren. Good luck...", ColorPair::new(WHITE, GREY10))
       );

       State {
           proc: true,
           refresh: true,
           passed_turn: false,
           go_next_level: false,
           status: ContextState::InGame,
           turn_state: TurnState::Player,
           world: World::new_game(),
           logs
       }
    }
    pub fn set_proc(&mut self) { self.proc = true }
    pub fn set_refresh(&mut self) { self.refresh = true }
}
impl GameState for State {
    fn tick(&mut self, con: &mut BTerm) {
        if self.turn_state == TurnState::Player { player_input(self, con) }
        else if self.turn_state == TurnState::GameOver { /*Do the other thing once it's ready*/ }

        match self.status {
            ContextState::MainMenu => {}
            ContextState::Paused => {}
            ContextState::GameMenu => {}
            ContextState::InGame => {
                exec_all_systems(self);
            }
        }

        if self.refresh {
            render_loop(&self, con);
            self.refresh = false;
        }
    }
}


fn exec_all_systems(gs: &mut State) {
    if gs.proc {
        //Execute the systems and shit
        process_fov(&mut gs.world.objects, &mut gs.world.map);
        update_blocked_tiles(&mut gs.world.objects, &mut gs.world.map, gs.world.depth);
        check_player_collisions(gs);

        if gs.passed_turn {
            gs.turn_state == TurnState::AI;
            gs.passed_turn = false;
            process_fov(&mut gs.world.objects, &mut gs.world.map);
        }

        if gs.turn_state == TurnState::AI {
            //Do all the ai stuff
            process_fov(&mut gs.world.objects, &mut gs.world.map);
            update_blocked_tiles(&mut gs.world.objects, &mut gs.world.map, gs.world.depth);
            gs.turn_state = TurnState::Player;
        }

        update_player_memory(&mut gs.world.objects);

        gs.proc = false;
    }
}


pub struct World {
    pub rng: RandomNumberGenerator,
    pub objects: Vec<Object>,
    pub map: Map,
    pub depth: u32,
    pub camera: Camera
}
impl World {
    pub fn empty() -> World {
        World {
            rng: RandomNumberGenerator::new(),
            objects: Vec::new(),
            map: Map::new(0,0),
            depth: 0,
            camera: Camera::new(Point::zero()),
        }
    }
    pub fn new_game() -> World {
        let mut rng = RandomNumberGenerator::new();
        let mut objects = Vec::new();
        let mut map = cellular_automata_builder(80,80, true);
        let camera = Camera::new(map.starting_pos.clone());
        objects.push(spawn_player(map.starting_pos.clone()));

        for _ in 1..=24 {
            let max_roll = map.valid_spawns.len() - 1;
            let index = rng.range(0, max_roll);
            let pos = map.valid_spawns[index].clone();
            objects.push(spawn_band_of_forsaken(&mut rng, pos, 1));
            map.valid_spawns.remove(index);
        }

        for _ in 1..=10 {
            let max_roll = map.valid_spawns.len() - 1;
            let index = rng.range(0, max_roll);
            let pos = map.valid_spawns[index].clone();
            objects.push(spawn_elf_pickup(&mut rng, pos, 1));
            map.valid_spawns.remove(index);
        }

        let mut world = World {
            rng,
            objects,
            map,
            depth: 1,
            camera
        };

        return world
    }
}