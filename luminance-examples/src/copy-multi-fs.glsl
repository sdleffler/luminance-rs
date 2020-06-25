in vec2 v_uv;

out vec4 frag;

uniform sampler2D source_texture_color;
uniform sampler2D source_texture_white;

void main() {
  frag *= 0;
  vec2 shift = vec2(0.25, 0.0);
  frag += vec4(texture(source_texture_color, v_uv + shift).rgb, 1.);
  frag += vec4(texture(source_texture_white, v_uv - shift).rgb, 1.);

  frag = pow(frag, vec4(1./2.2));
}
