#version 450

struct Instance {
	mat4 transform;
	vec4 color;
};

layout(set = 0, binding = 0) restrict readonly buffer Instances {
	Instance instances[];
};

layout(set = 0, binding = 1) uniform Camera {
	mat4 view;
	mat4 proj;
} camera;

layout(location = 0) out vec2 outUV;
layout(location = 1) out vec4 outColor;

/// 四角形を描画するバーテックスシェーダ
///
/// 頂点インデックスから四角形を構築する。
///
/// NOTE: このシェーダを使う場合、
///       必ず`ash::Device::draw()`を用い、かつ第2引数(`vertex_count`)に`4`を指定すること。
void main() {
	vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
	outUV    = uv;
	outColor = instances[gl_InstanceIndex].color;
	gl_Position = camera.proj
		* camera.view
		* instances[gl_InstanceIndex].transform
		* vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}
