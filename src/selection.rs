use std::collections::HashSet;

use super::*;

fn change_outline(
    materials: &mut ResMut<Assets<OutlineMaterial>>,
    handles: &Query<&Handle<OutlineMaterial>>,
    entities: &HashSet<Entity>,
    with_outline: bool,
) {
    for &entity in entities.iter() {
        if let Ok(handle) = handles.get_component::<Handle<OutlineMaterial>>(entity) {
            let mut material = materials.get_mut(handle).unwrap();
            material.with_outline = with_outline;
        }
    }
}

pub fn show_selection_system(
    mut state: Local<EventReader<CursorSelectionEvent>>,
    selection: Res<CursorSelection>,
    selection_changed_events: Res<Events<CursorSelectionEvent>>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    handles: Query<&Handle<OutlineMaterial>>,
    mut texts: Query<Mut<Text>>,
) {
    for event in state.iter(&selection_changed_events) {
        change_outline(&mut materials, &handles, &event.prev_enemies, false);
        change_outline(&mut materials, &handles, &event.prev_loots, false);
        change_outline(&mut materials, &handles, &selection.enemies, true);
        change_outline(&mut materials, &handles, &selection.loots, true);
        // Quick/dirty UI selection
        let value = if selection.enemies.len() > 0 {
            "Frail target"
        } else if selection.loots.len() > 0 {
            "Uber Loot"
        } else {
            ""
        };
        if let Some(mut text) = texts.iter_mut().next() {
            text.value = value.to_string();
        }
    }
}
