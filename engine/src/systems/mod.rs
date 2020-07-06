mod app_control_systems;
pub use app_control_systems::quit_if_escape_pressed;

// mod update_resource_manager;
// pub use update_resource_manager::update_resource_manager;

mod add_light_debug_draw;
pub use add_light_debug_draw::add_light_debug_draw;

mod temp_logic;
pub use temp_logic::imgui_draw_mouse_coordinates;

use minimum::systems::*;

use legion::prelude::*;

use minimum::editor::resources::EditorMode;
use fnv::FnvHashMap;
use minimum::resources::TimeResource;
use minimum::resources::editor::EditorStateResource;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ScheduleCriteria {
    is_simulation_paused: bool,
    editor_mode: EditorMode,
}

impl ScheduleCriteria {
    pub fn new(
        is_simulation_paused: bool,
        editor_mode: EditorMode,
    ) -> Self {
        ScheduleCriteria {
            is_simulation_paused,
            editor_mode,
        }
    }
}

struct ScheduleBuilder<'a> {
    criteria: &'a ScheduleCriteria,
    schedule: legion::systems::schedule::Builder,
}

impl<'a> ScheduleBuilder<'a> {
    fn new(criteria: &'a ScheduleCriteria) -> Self {
        ScheduleBuilder::<'a> {
            criteria,
            schedule: Default::default(),
        }
    }

    fn build(self) -> Schedule {
        self.schedule.build()
    }

    fn always<F>(
        mut self,
        f: F,
    ) -> Self
    where
        F: Fn() -> Box<dyn Schedulable>,
    {
        self.schedule = self.schedule.add_system((f)());
        self
    }

    fn editor_only<F>(
        mut self,
        f: F,
    ) -> Self
    where
        F: Fn() -> Box<dyn Schedulable>,
    {
        if self.criteria.editor_mode == EditorMode::Active {
            self.schedule = self.schedule.add_system((f)());
        }

        self
    }

    fn simulation_unpaused_only<F>(
        mut self,
        f: F,
    ) -> Self
    where
        F: Fn() -> Box<dyn Schedulable>,
    {
        if !self.criteria.is_simulation_paused {
            self.schedule = self.schedule.add_system((f)());
        }

        self
    }

    fn always_thread_local<F: FnMut(&mut World, &mut Resources) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.schedule = self.schedule.add_thread_local_fn(f);
        self
    }

    fn flush(mut self) -> Self {
        self.schedule = self.schedule.flush();
        self
    }
}

// pub fn create_draw_schedule(criteria: &ScheduleCriteria) -> Schedule {
//     ScheduleBuilder::new(criteria).always(draw).build()
// }

pub struct ScheduleManager {
    update_schedules: FnvHashMap<ScheduleCriteria, Schedule>,
}

impl ScheduleManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // The expected states for which we will generate schedules
        let expected_criteria = Self::create_schedule_criteria();

        // Populate a lookup for the schedules.. on each update/draw, we will check the current
        // state of the application, create an appropriate ScheduleCriteria, and use it to look
        // up the correct schedule to run
        let mut update_schedules = FnvHashMap::default();

        for criteria in &expected_criteria {
            update_schedules.insert(criteria.clone(), Self::create_update_schedule(&criteria));
        }

        ScheduleManager { update_schedules }
    }

    pub fn update(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) {
        let current_criteria = Self::get_current_schedule_criteria(resources);
        let schedule = self.update_schedules.get_mut(&current_criteria).unwrap();
        schedule.execute(world, resources);
    }

    // Determine the current state of the game
    fn get_current_schedule_criteria(resources: &Resources) -> ScheduleCriteria {
        ScheduleCriteria::new(
            resources
                .get::<TimeResource>()
                .unwrap()
                .is_simulation_paused(),
            resources
                .get::<EditorStateResource>()
                .unwrap()
                .editor_mode(),
        )
    }

    fn create_schedule_criteria() -> Vec<ScheduleCriteria> {
        vec![
            ScheduleCriteria::new(false, EditorMode::Inactive),
            ScheduleCriteria::new(true, EditorMode::Active),
        ]
    }

    fn create_update_schedule(criteria: &ScheduleCriteria) -> Schedule {
        use minimum::editor::systems::*;

        ScheduleBuilder::new(criteria)
            .always(update_input_resource)
            .always(advance_time)
            .always(quit_if_escape_pressed)
            .always_thread_local(update_asset_manager)
            //.always(update_resource_manager)
            .always(add_light_debug_draw)
            //.always(imgui_draw_mouse_coordinates)
            //.always(update_fps_text)
            //.always(update_physics)
            //.simulation_unpaused_only(read_from_physics)
            // --- Editor stuff here ---
            // Prepare to handle editor input
            .always_thread_local(editor_refresh_selection_world)
            // Editor input
            .always_thread_local(reload_editor_state_if_file_changed)
            .always(editor_keybinds)
            .always(editor_mouse_input)
            .always(editor_update_editor_draw)
            .always(editor_gizmos)
            .always(editor_handle_selection)
            .always(editor_imgui_menu)
            .always(editor_entity_list_window)
            .always_thread_local(editor_inspector_window)
            // Editor processing
            .always_thread_local(editor_process_edit_diffs)
            .always_thread_local(editor_process_selection_ops)
            .always_thread_local(editor_process_editor_ops)
            // Editor output
            .always(draw_selection_shapes) //TODO: Requires pushing 3d debug draw down
            // --- End editor stuff ---
            .always(input_reset_for_next_frame)
            .build()
    }
}
