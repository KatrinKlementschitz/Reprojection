#version 450

uniform sampler3D block_tex;
uniform sampler2D frame_tex;
uniform uint width, height, depth;
uniform bool show_texture;

uniform mat4 prev_view;
uniform mat4 prev_proj;

in vec2 tex_pos;
in vec3 ray_dir;
in vec3 ray_pos;
in mat4 o_proj;
in mat4 o_prev_view;
out vec4 f_color;

const float VOXEL_SIZE = 1;
const float u_max_steps = 500;
int steps = 0;

struct hit_record {
    float t;
    vec3 position;
    vec3 normal;
    uint id;
};

struct ray {
    vec3 position;
    vec3 direction;
};

vec3 point_at(ray r, float t) {
    return r.position + t * r.direction;
}

bool insideBounds(vec3 pos)
{
    return pos.x >= 0 && pos.x < float(width) && pos.z >= 0 && pos.z < float(height) && pos.y > 0 && pos.y < float(depth);
}

uint getBlock(vec3 pos)
{
    uint block = 0;

    if (insideBounds(pos))
    {
        pos.x /= width;
        pos.z /= height;
        pos.y /= depth;
        block = uint(texture(block_tex, pos.xzy).x*255.0);
    }

    return block;
}

void voxel(float voxel_size, vec3 direction, ivec3 current_voxel, vec3 position, out ivec3 step_, out vec3 next_boundary, out vec3 t_max, out vec3 t_delta)
{
    step_ = ivec3(
        (direction.x > 0.0) ? 1 : -1,
        (direction.y > 0.0) ? 1 : -1,
        (direction.z > 0.0) ? 1 : -1
    );
    next_boundary = vec3(
        float((step_.x > 0) ? current_voxel.x + 1 : current_voxel.x) * voxel_size,
        float((step_.y > 0) ? current_voxel.y + 1 : current_voxel.y) * voxel_size,
        float((step_.z > 0) ? current_voxel.z + 1 : current_voxel.z) * voxel_size
    );
    t_max = (next_boundary - position) / direction; // we will move along the axis with the smallest value
    t_delta = voxel_size / direction * vec3(step_);
}

hit_record voxel_traversal(in ray r) {
    hit_record record;
    r.direction = normalize(r.direction);
    ivec3 current_voxel = ivec3(floor(r.position / VOXEL_SIZE));

    ivec3 step_;
    vec3 next_boundary;
    vec3 t_max;
    vec3 t_delta;
    voxel(VOXEL_SIZE, r.direction, current_voxel, r.position, step_, next_boundary, t_max, t_delta);

    do {
        if (t_max.x < t_max.y && t_max.x < t_max.z) {
            record.t = t_max.x;
            record.normal = vec3(float(-step_.x), 0.0, 0.0);
            t_max.x += t_delta.x;
            current_voxel.x += step_.x;
        }
        else if (t_max.y < t_max.z) {
            record.t = t_max.y;
            record.normal = vec3(0.0, float(-step_.y), 0.0);
            t_max.y += t_delta.y;
            current_voxel.y += step_.y;
        }
        else {
            record.t = t_max.z;
            record.normal = vec3(0.0, 0.0, float(-step_.z));
            t_max.z += t_delta.z;
            current_voxel.z += step_.z;
        }

        record.id = getBlock(current_voxel);
        record.position = point_at(r, record.t);

        if (record.id != 0u) { return record; }
        steps++;
    } while (steps < u_max_steps);
    return record;
}

void main() {

    ray r;  
    r.position = ray_pos;
    r.direction = ray_dir;
    hit_record rec = voxel_traversal(r);

    if(show_texture) {
        vec4 point = prev_proj * prev_view * vec4(rec.position, 1.0);
        vec3 p = point.xyz / point.w; // perspective division

        vec2 previous_uv = p.xy/2.0 + vec2(0.5);
        f_color = texture(frame_tex, previous_uv);

        return;
    }
    
    if(rec.id != 0)
        f_color = vec4(vec3(float(rec.id)/255.0), 1.0);
    else
        f_color = vec4(ray_dir, 1.0);
}