use renderer::nodes::{RenderPhaseIndex, SubmitNode};
use renderer::nodes::RenderPhase;
use std::convert::TryInto;

renderer::declare_render_phase!(
    PreUiRenderPhase,
    PRE_UI_RENDER_PHASE_INDEX,
    pre_ui_render_phase_sort_submit_nodes
);

fn pre_ui_render_phase_sort_submit_nodes(mut submit_nodes: Vec<SubmitNode>) -> Vec<SubmitNode> {
    // Sort by feature
    log::trace!("Sort phase {}", PreUiRenderPhase::render_phase_debug_name());
    submit_nodes.sort_unstable_by(|a, b| a.feature_index().cmp(&b.feature_index()));

    submit_nodes
}
