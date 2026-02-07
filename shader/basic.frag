#version 450

layout(set = 1, binding = 0) uniform texture2D texs[1];
layout(set = 1, binding = 1) uniform sampler smplr;

layout(location = 0) in flat uint inTexId;
layout(location = 1) in vec2 inUV;
layout(location = 2) in vec4 inColor;

layout(location = 0) out vec4 outColor;

void main() {
	outColor = texture(sampler2D(texs[inTexId], smplr), inUV) * inColor;
}
