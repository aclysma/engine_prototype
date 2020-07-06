use minimum_sdl2::imgui::Sdl2ImguiManager;
use renderer::vulkan::{VkSurface, Window, VkDeviceContext, VkContext, FrameInFlight};
use ash::prelude::VkResult;
use std::mem::ManuallyDrop;
use ash::vk;
use minimum::resources::{AssetResource, TimeResource, ViewportResource};
use renderer::assets::resources::{ResourceManager, ResourceArc, ImageViewResource};
use crate::features::debug3d::create_debug3d_extract_job;
use crate::features::sprite::{SpriteRenderNodeSet, create_sprite_extract_job};
use renderer::visibility::{StaticVisibilityNodeSet, DynamicVisibilityNodeSet};
use renderer::nodes::{
    RenderPhaseMaskBuilder, RenderPhaseMask, RenderRegistry, RenderViewSet, AllRenderNodes,
    FramePacketBuilder, ExtractJobSet,
};
use crate::phases::{OpaqueRenderPhase, UiRenderPhase, PreUiRenderPhase};
use crate::phases::TransparentRenderPhase;
use legion::prelude::*;
use crate::render_contexts::{RenderJobExtractContext};
use crate::features::mesh::{create_mesh_extract_job, MeshRenderNodeSet};
use std::sync::{Arc, Mutex};

mod static_resources;
use static_resources::GameRendererStaticResources;

mod render_thread;
use render_thread::RenderThread;

mod swapchain_resources;
use swapchain_resources::SwapchainResources;

mod render_frame_job;
use render_frame_job::RenderFrameJob;

//TODO: Find a way to not expose this
mod swapchain_handling;
pub use swapchain_handling::SwapchainLifetimeListener;
use ash::version::DeviceV1_0;
use crate::features::imgui::create_imgui_extract_job;

pub struct GameRendererInner {
    imgui_font_atlas_image_view: ResourceArc<ImageViewResource>,

    static_resources: GameRendererStaticResources,
    swapchain_resources: Option<SwapchainResources>,

    main_camera_render_phase_mask: RenderPhaseMask,

    previous_frame_result: Option<VkResult<()>>,

    render_thread: RenderThread,
}

#[derive(Clone)]
pub struct GameRenderer {
    inner: Arc<Mutex<GameRendererInner>>,
}

impl GameRenderer {
    pub fn new(
        _window: &dyn Window,
        resources: &Resources,
    ) -> VkResult<Self> {
        let game_renderer_resources =
            GameRendererStaticResources::new(resources)?;

        log::info!("all waits complete");

        let mut asset_resource_fetch = resources.get_mut::<AssetResource>().unwrap();
        let asset_resource = &mut *asset_resource_fetch;

        let mut resource_manager_fetch = resources.get_mut::<ResourceManager>().unwrap();
        let mut resource_manager = &mut *resource_manager_fetch;

        let vk_context = resources.get_mut::<VkContext>().unwrap();
        let device_context = vk_context.device_context();

        let imgui_font_atlas_image_view = GameRenderer::create_font_atlas_image_view(
            &device_context,
            &mut resource_manager,
            resources,
        )?;

        let main_camera_render_phase_mask = RenderPhaseMaskBuilder::default()
            .add_render_phase::<OpaqueRenderPhase>()
            .add_render_phase::<TransparentRenderPhase>()
            .add_render_phase::<PreUiRenderPhase>()
            .add_render_phase::<UiRenderPhase>()
            .build();

        let render_thread = RenderThread::start();

        let renderer = GameRendererInner {
            imgui_font_atlas_image_view,
            static_resources: game_renderer_resources,
            swapchain_resources: None,

            main_camera_render_phase_mask,

            render_thread,

            previous_frame_result: Some(Ok(())),
        };

        Ok(GameRenderer {
            inner: Arc::new(Mutex::new(renderer)),
        })
    }

    fn create_font_atlas_image_view(
        device_context: &VkDeviceContext,
        resource_manager: &mut ResourceManager,
        resources: &Resources,
    ) -> VkResult<ResourceArc<ImageViewResource>> {
        //TODO: Simplify this setup code for the imgui font atlas
        let imgui_font_atlas = resources
            .get::<Sdl2ImguiManager>()
            .unwrap()
            .build_font_atlas();

        let imgui_font_atlas = renderer::assets::image_utils::DecodedTexture {
            width: imgui_font_atlas.width,
            height: imgui_font_atlas.height,
            data: imgui_font_atlas.data,
            color_space: renderer::assets::image_utils::ColorSpace::Linear,
            mips: renderer::assets::image_utils::default_mip_settings_for_image(
                imgui_font_atlas.width,
                imgui_font_atlas.height,
            ),
        };

        let mut imgui_font_atlas_image = renderer::assets::image_utils::load_images(
            &device_context,
            device_context
                .queue_family_indices()
                .transfer_queue_family_index,
            &device_context.queues().transfer_queue,
            device_context
                .queue_family_indices()
                .graphics_queue_family_index,
            &device_context.queues().graphics_queue,
            &[imgui_font_atlas],
        )?;

        let dyn_resource_allocator = resource_manager.create_dyn_resource_allocator_set();
        let imgui_font_atlas_image = dyn_resource_allocator
            .insert_image(unsafe { ManuallyDrop::take(&mut imgui_font_atlas_image[0]) });

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(imgui_font_atlas_image.get_raw().image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .subresource_range(*subresource_range);

        let imgui_font_atlas_image_view = unsafe {
            device_context
                .device()
                .create_image_view(&image_view_info, None)?
        };

        let imgui_font_atlas_image_view = dyn_resource_allocator
            .insert_image_view(imgui_font_atlas_image, imgui_font_atlas_image_view);

        Ok(imgui_font_atlas_image_view)
    }
}

impl GameRenderer {
    // This is externally exposed, it checks result of the previous frame (which implicitly also
    // waits for the previous frame to complete if it hasn't already)
    pub fn begin_render(
        &self,
        resources: &Resources,
        world: &World,
        window: &dyn Window,
    ) -> VkResult<()> {
        let t0 = std::time::Instant::now();
        // This lock will delay until the previous frame completes being submitted to GPU
        resources
            .get_mut::<VkSurface>()
            .unwrap()
            .wait_until_frame_not_in_flight()?;
        let t1 = std::time::Instant::now();
        log::trace!(
            "[main] wait for previous frame present {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        // Here, we error check from the previous frame. This includes checking for errors that happened
        // during setup (i.e. before we finished building the frame job). So
        {
            let result = self.inner.lock().unwrap().previous_frame_result.take();
            if let Some(result) = result {
                if let Err(e) = result {
                    match e {
                        ash::vk::Result::ERROR_OUT_OF_DATE_KHR => {
                            SwapchainLifetimeListener::rebuild_swapchain(resources, window, self)
                        }
                        ash::vk::Result::SUCCESS => Ok(()),
                        ash::vk::Result::SUBOPTIMAL_KHR => Ok(()),
                        //ash::vk::Result::TIMEOUT => Ok(()),
                        _ => {
                            log::warn!("Unexpected rendering error");
                            return Err(e);
                        }
                    }?;
                }
            }
        }

        // If we get an error before kicking off rendering, stash it for the next frame. We could
        // consider acting on it instead, but for now lets just have a single consistent codepath
        if let Err(e) = self.do_begin_render(resources, world, window) {
            log::warn!("Received error immediately from do_begin_render: {:?}", e);
            self.inner.lock().unwrap().previous_frame_result = Some(Err(e));
        }

        Ok(())
    }

    //TODO: In a failure, return the frame_in_flight and cancel the render. This will make
    // previous_frame_result unnecessary
    pub fn do_begin_render(
        &self,
        resources: &Resources,
        world: &World,
        window: &dyn Window,
    ) -> VkResult<()> {
        // Fetch the next swapchain image
        let frame_in_flight = {
            let mut surface = resources.get_mut::<VkSurface>().unwrap();
            let t0 = std::time::Instant::now();
            let result = surface.acquire_next_swapchain_image(window);
            let t1 = std::time::Instant::now();
            log::trace!(
                "[main] wait for swapchain image took {} ms",
                (t1 - t0).as_secs_f32() * 1000.0
            );
            result?
        };

        // Get command buffers to submit
        Self::render(self, world, resources, window, frame_in_flight)
    }

    pub fn render(
        game_renderer: &GameRenderer,
        world: &World,
        resources: &Resources,
        _window: &dyn Window,
        frame_in_flight: FrameInFlight,
    ) -> VkResult<()> {
        let t0 = std::time::Instant::now();

        //
        // Fetch resources
        //

        let time_state_fetch = resources.get::<TimeResource>().unwrap();
        let time_resource = &*time_state_fetch;

        let static_visibility_node_set_fetch = resources.get::<StaticVisibilityNodeSet>().unwrap();
        let static_visibility_node_set = &*static_visibility_node_set_fetch;

        let dynamic_visibility_node_set_fetch =
            resources.get::<DynamicVisibilityNodeSet>().unwrap();
        let dynamic_visibility_node_set = &*dynamic_visibility_node_set_fetch;

        let mut viewport = resources.get_mut::<ViewportResource>().unwrap();

        let render_registry = resources.get::<RenderRegistry>().unwrap().clone();
        let device_context = resources.get::<VkDeviceContext>().unwrap().clone();

        let mut resource_manager_fetch = resources.get_mut::<ResourceManager>().unwrap();
        let resource_manager = &mut *resource_manager_fetch;

        // Call this here - represents that the previous frame was completed
        resource_manager.on_frame_complete()?;

        let mut guard = game_renderer.inner.lock().unwrap();
        let main_camera_render_phase_mask = guard.main_camera_render_phase_mask.clone();
        let swapchain_resources = guard.swapchain_resources.as_mut().unwrap();
        let swapchain_surface_info = swapchain_resources.swapchain_surface_info.clone();

        // https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        let vulkan_projection_correction = glam::Mat4::from_scale(glam::Vec3::new(1.0, -1.0, 0.5)) *
            glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 1.0));

        //
        // View Management
        //
        let render_view_set = RenderViewSet::default();
        let (main_view, view_proj) = {
            let camera_rotate_speed = 1.0;
            let camera_distance_multiplier = 1.0;
            let loop_time = time_resource.simulation_time.total_time().as_secs_f32();
            let eye = glam::Vec3::new(
                camera_distance_multiplier * 8.0 * f32::cos(camera_rotate_speed * loop_time / 2.0),
                camera_distance_multiplier * 8.0 * f32::sin(camera_rotate_speed * loop_time / 2.0),
                camera_distance_multiplier * 5.0,
            );

            let extents_width = swapchain_surface_info.extents.width;
            let extents_height = swapchain_surface_info.extents.height;
            let aspect_ratio = extents_width as f32 / extents_height as f32;

            let near_clip = 0.1;
            let far_clip = 25.0;
            let fov = std::f32::consts::FRAC_PI_4;
            let up = glam::Vec3::new(0.0, 0.0, 1.0);
            let dir = (glam::Vec3::new(0.0, 0.0, 0.0) - eye).normalize();

            let view = glam::Mat4::look_at_rh(
                eye,
                glam::Vec3::new(0.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 0.0, 1.0),
            );
            let proj = vulkan_projection_correction * glam::Mat4::perspective_rh_gl(
                fov,
                aspect_ratio,
                near_clip,
                far_clip,
            );

            let view_proj = proj * view;

            let main_view = render_view_set.create_view(
                eye,
                view,
                proj,
                main_camera_render_phase_mask,
                "main".to_string(),
            );

            viewport.set_world_space_view(
                proj,
                view,
                // eye,
                // dir,
                // up,
                // fov,
                // near_clip,
                // far_clip
            );

            (main_view, view_proj)
        };

        // Set up the screen-space viewport matrices
        {
            let multiplier = 600.0 as f32 / swapchain_surface_info.extents.height as f32;

            let half_extents_width = (swapchain_surface_info.extents.width as f32 * multiplier) / 2.0;
            let half_extents_height = (swapchain_surface_info.extents.height as f32 * multiplier) / 2.0;

            // let view = glam::Mat4::look_at_rh(
            //     glam::Vec3::new(0.0, 0.0, -100.0),
            //     glam::Vec3::new(0.0, 0.0, 0.0),
            //     glam::Vec3::new(0.0, 1.0, 0.0)
            // );
            let proj = vulkan_projection_correction * glam::Mat4::orthographic_rh_gl(
                -half_extents_width,
                half_extents_width,
                -half_extents_height,
                half_extents_height,
                -100.0,
                100.0
            );

            viewport.set_screen_space_view(proj /* * view*/);
        }

        viewport.set_viewport_size_in_pixels(glam::Vec2::new(
            swapchain_surface_info.extents.width as f32,
            swapchain_surface_info.extents.height as f32
        ));

        //
        // Visibility
        //
        let main_view_static_visibility_result =
            static_visibility_node_set.calculate_static_visibility(&main_view);
        let main_view_dynamic_visibility_result =
            dynamic_visibility_node_set.calculate_dynamic_visibility(&main_view);

        log::trace!(
            "main view static node count: {}",
            main_view_static_visibility_result.handles.len()
        );

        log::trace!(
            "main view dynamic node count: {}",
            main_view_dynamic_visibility_result.handles.len()
        );

        let sprite_render_nodes = resources.get::<SpriteRenderNodeSet>().unwrap();
        let mesh_render_nodes = resources.get::<MeshRenderNodeSet>().unwrap();
        let mut all_render_nodes = AllRenderNodes::new();
        all_render_nodes.add_render_nodes(&*sprite_render_nodes);
        all_render_nodes.add_render_nodes(&*mesh_render_nodes);

        let frame_packet_builder = FramePacketBuilder::new(&all_render_nodes);

        // After these jobs end, user calls functions to start jobs that extract data
        frame_packet_builder.add_view(
            &main_view,
            &[
                main_view_static_visibility_result,
                main_view_dynamic_visibility_result,
            ],
        );

        let mut descriptor_set_allocator = resource_manager.create_descriptor_set_allocator();
        swapchain_resources
            .debug_material_per_frame_data
            .set_buffer_data(0, &view_proj);
        swapchain_resources
            .debug_material_per_frame_data
            .flush(&mut descriptor_set_allocator)?;
        descriptor_set_allocator.flush_changes()?;

        //
        // Update Resources and flush descriptor set changes
        //
        resource_manager.on_begin_frame()?;

        //
        // Extract Jobs
        //
        let frame_packet = frame_packet_builder.build();
        let extract_job_set = {
            let sprite_pipeline_info = resource_manager.get_pipeline_info(
                &guard.static_resources.sprite_material,
                &swapchain_surface_info,
                0,
            );

            let mesh_pipeline_info = resource_manager.get_pipeline_info(
                &guard.static_resources.mesh_material,
                &swapchain_surface_info,
                0,
            );

            let debug3d_pipeline_info = resource_manager.get_pipeline_info(
                &guard.static_resources.debug3d_material,
                &swapchain_surface_info,
                0,
            );

            let debug3d_pipeline_info_no_depth = resource_manager.get_pipeline_info(
                &guard.static_resources.debug3d_material_no_depth,
                &swapchain_surface_info,
                0,
            );

            let imgui_pipeline_info = resource_manager.get_pipeline_info(
                &guard.static_resources.imgui_material,
                &swapchain_surface_info,
                0,
            );

            let mut extract_job_set = ExtractJobSet::new();

            // Sprites
            extract_job_set.add_job(create_sprite_extract_job(
                device_context.clone(),
                resource_manager.create_descriptor_set_allocator(),
                sprite_pipeline_info,
                &guard.static_resources.sprite_material,
            ));

            // Meshes
            extract_job_set.add_job(create_mesh_extract_job(
                resource_manager.create_descriptor_set_allocator(),
                mesh_pipeline_info,
                &guard.static_resources.mesh_material,
            ));

            // Debug 3D
            extract_job_set.add_job(create_debug3d_extract_job(
                device_context.clone(),
                resource_manager.create_descriptor_set_allocator(),
                debug3d_pipeline_info,
                debug3d_pipeline_info_no_depth,
                &guard.static_resources.debug3d_material,
            ));

            extract_job_set.add_job(create_imgui_extract_job(
                device_context.clone(),
                resource_manager.create_descriptor_set_allocator(),
                imgui_pipeline_info,
                swapchain_surface_info.extents,
                &guard.static_resources.imgui_material,
                //guard.imgui_font_atlas.clone(),
                guard.imgui_font_atlas_image_view.clone(),
            ));

            extract_job_set
        };

        let mut extract_context =
            RenderJobExtractContext::new(&world, &resources, resource_manager);
        let prepare_job_set =
            extract_job_set.extract(&mut extract_context, &frame_packet, &[&main_view]);

        let opaque_pipeline_info = resource_manager.get_pipeline_info(
            &guard.static_resources.sprite_material,
            &swapchain_surface_info,
            0,
        );

        let imgui_pipeline_info = resource_manager.get_pipeline_info(
            &guard.static_resources.imgui_material,
            &swapchain_surface_info,
            0,
        );

        let dyn_resource_allocator_set = resource_manager.create_dyn_resource_allocator_set();

        let t1 = std::time::Instant::now();
        log::trace!(
            "[main] render extract took {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        let game_renderer = game_renderer.clone();

        let prepared_frame = RenderFrameJob {
            game_renderer,
            prepare_job_set,
            dyn_resource_allocator_set,
            frame_packet,
            main_view,
            render_registry: render_registry.clone(),
            device_context: device_context.clone(),
            opaque_pipeline_info,
            imgui_pipeline_info,
            frame_in_flight,
        };

        guard.render_thread.render(prepared_frame);

        Ok(())
    }
}
