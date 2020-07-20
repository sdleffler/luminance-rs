uniform mat4 view;
uniform float fovy;
uniform float aspect_ratio;

out vec3 view_dir;

vec2[4] POSITIONS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2(-1.,  1.),
  vec2( 1.,  1.)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexID], 0., 1.);

  float fovy_2 = fovy * .5;

  vec3 dir = vec3(gl_Position.x, gl_Position.y, -1. / tan(fovy_2));

  view_dir = (view * vec4(dir, 1.)).xyz;

  // correct aspect ratio
  gl_Position.y *= aspect_ratio;
}
