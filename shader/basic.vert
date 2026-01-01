#version 450

layout(location = 0) out vec2 outUV;

/// 四角形を描画するバーテックスシェーダ
///
/// 頂点インデックスから四角形を構築する。
///
/// NOTE: このシェーダを使う場合、
///       必ず`ash::Device::draw()`を用い、かつ第2引数(`vertex_count`)に`4`を指定すること。
void main() {
	vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
	outUV = uv;
	gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}
