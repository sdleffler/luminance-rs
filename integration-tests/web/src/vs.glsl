in vec2 co;

uniform mat4 translation_mat;

void main() {
  gl_Position = translation_mat * vec4(co, 0., 1.);
}
