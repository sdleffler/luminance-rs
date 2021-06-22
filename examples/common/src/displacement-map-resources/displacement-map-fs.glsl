in vec2 v_uv;
out vec4 fragment;

uniform sampler2D image;
uniform sampler2D displacement_map_1;
uniform sampler2D displacement_map_2;
uniform vec2 window_dimensions;
uniform float displacement_scale;
uniform float time;

void main() {
    vec4 displacement_1 = texture(
        displacement_map_1,
        mod(window_dimensions * v_uv, 128.0) / 128.0
    ) - 0.5;
    float scale_1 = sin(time * 3.141592) * displacement_scale;
    vec4 displacement_2 = texture(
        displacement_map_2,
        mod(window_dimensions * v_uv, 101.0) / 101.0
    ) - 0.5;
    displacement_2.y *= -1.0;
    float scale_2 = cos(time * 4.0 * 3.141592) * displacement_scale / 2.0;
    vec2 new_uv = mod(v_uv + scale_1 * displacement_1.r + scale_2 * displacement_2.r, 1.0);

    fragment = texture(image, new_uv);
}
