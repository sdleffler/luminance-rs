in vec3 view_dir;

out vec3 frag;

uniform samplerCube skybox;

void main() {
  vec3 cube_color = texture(skybox, view_dir).rgb;
  frag = cube_color;
}
