#version 450
layout (location = 0) in vec3 position;
uniform mat4 proj;
uniform mat4 view;
out vec2 tex_pos;
out vec3 ray_dir;
out vec3 ray_pos;

void main() {
    mat4 inv_view = inverse(view);

    gl_Position = vec4(position.x, position.y, 1.0, 1.0);
    tex_pos = position.xy/2.0 + vec2(0.5);
    ray_dir = transpose(mat3(view)) * (inverse(proj) * gl_Position).xyz;
    ray_pos = vec3(inv_view[0][3], inv_view[1][3], inv_view[2][3]);
}