#version 450

#define MAX_SOFTBODY_VERT 12

layout(binding = 0) uniform shared_ubo {
  mat4 view;
  mat4 projection;
} shared_data;

layout(location = 0) in  vec3 inPosition;
layout(location = 1) in  vec3 inColor;
layout(location = 0) out vec3 fragColor;

out gl_PerVertex {
  vec4 gl_Position;
};

void main() {
  gl_Position = shared_data.projection * shared_data.view
    * vec4(inPosition, 1);

  fragColor = inColor;
}
