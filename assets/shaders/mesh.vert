#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(set = 2, binding = 0) uniform PerObjectData {
    mat4 model_view;
    mat4 model_view_proj;
} per_object_data;

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_normal;
// w component is a sign value (-1 or +1) indicating handedness of the tangent basis
// see GLTF spec for more info
layout (location = 2) in vec4 in_tangent;
layout (location = 3) in vec2 in_uv;

// Do all math in view space so that it is more easily portable to deferred/clustered
// forward rendering (vs = view space)
layout (location = 0) out vec3 out_position_vs;
layout (location = 1) out vec3 out_normal_vs;
layout (location = 2) out vec3 out_tangent_vs;
layout (location = 3) out vec3 out_binormal_vs;
layout (location = 4) out vec2 out_uv;

void main() {
    gl_Position = per_object_data.model_view_proj * vec4(in_pos, 1.0);
    out_position_vs = (per_object_data.model_view * vec4(in_pos, 1.0)).xyz;

    // NOTE: Not sure if I need to normalize after the matrix multiply
    out_normal_vs = mat3(per_object_data.model_view) * in_normal;
    out_tangent_vs = mat3(per_object_data.model_view) * in_tangent.xyz;
    vec3 binormal = cross(in_normal, in_tangent.xyz) * in_tangent.w;
    out_binormal_vs = mat3(per_object_data.model_view) * binormal;

    out_uv = in_uv;
}
