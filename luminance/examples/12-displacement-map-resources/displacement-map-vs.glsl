out vec2 v_uv;

const vec2[4] QUAD_POSITION = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0)
);

void main() {
    vec2 position = QUAD_POSITION[gl_VertexID];
    gl_Position = vec4(position, 0.0, 1.0);
    v_uv = position * 0.5 + 0.5;
}
