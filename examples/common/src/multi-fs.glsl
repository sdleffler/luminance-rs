in vec3 v_color;

layout (location = 0) out vec3 frag_color;
layout (location = 1) out float frag_white;

void main() {
  frag_color = v_color.rgb;
  frag_white = 1.;
}
