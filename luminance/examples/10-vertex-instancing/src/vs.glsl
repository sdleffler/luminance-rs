layout (location = 0) in vec2 co;
layout (location = 1) in vec3 color;
layout (location = 2) in vec2 position;
layout (location = 3) in float weight;

out vec3 v_color;

void main() {
  gl_Position = vec4(co * weight + position, 0., 1.);
  v_color = color;
}
