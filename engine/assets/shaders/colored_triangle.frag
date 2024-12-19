#version 450

//shader input
layout (location = 0) in vec3 inColor;
layout (location = 1) in vec2 inUV;

//output write
layout (location = 0) out vec4 outFragColor;

layout(set = 1, binding = 0) uniform sampler2D displayTexture;

// layout(buffer_reference, std430) readonly buffer MaterialBuffer{
// 	MaterialData materials[];
// };

//push constants block
// layout( push_constant ) uniform constants
// {
// 	VertexBuffer  vertex_buffer;
// 	MeshBuffer    mesh_data_buffer;
// 	MaterialData  material_buffer;
// 	uint          mesh_index;
// 	uint          material_index;
// } PushConstants; //todo

void main()
{
	outFragColor = texture(displayTexture, inUV);
}
