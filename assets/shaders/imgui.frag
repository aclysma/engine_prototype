#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (set = 0, binding = 1) uniform sampler smp;

layout (set = 1, binding = 0) uniform texture2D tex;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = texture(sampler2D(tex, smp), uv) * color;
}