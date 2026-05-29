use bevy_ecs::prelude::World;
use game_core::FactionReputation;
use game_world::{FactionRelationships, ReputationTracker, PLAYER_FACTION_ID};

pub fn sync_reputation_systems(ecs_world: &mut World) {
    let rep = ecs_world.get_resource::<ReputationTracker>().cloned();
    let rels = ecs_world.get_resource::<FactionRelationships>().cloned();
    let frep = ecs_world.get_resource_mut::<FactionReputation>();

    if let (Some(rep), Some(rels), Some(ref mut frep)) = (rep, rels, frep) {
        for ((fa, fb), &val) in rep.all() {
            let faction_pair = [fa, fb];
            for &fid in &faction_pair {
                if *fid == PLAYER_FACTION_ID { continue; }
                for (name, &id) in rels.name_id_pairs() {
                    if id == *fid {
                        frep.set(name, val * 100);
                        break;
                    }
                }
            }
        }
    }
}
