#version 450
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 outColor;
layout (location = 1) out vec2 outUV;

struct GlobalSceneData {
    mat4 view;
    //----------------- 16-byte boundary
    mat4 proj;
    //----------------- 16-byte boundary
    mat4 view_proj;
    //----------------- 16-byte boundary
    vec4 ambient_color;
    vec4 sunlight_dir;
    vec4 sunlight_color;
    vec4 padding0;
    //----------------- 16-byte boundary
};

struct Vertex {

	vec3 position;
	float uv_x;
	vec3 normal;
	float uv_y;
	vec4 color;
};

struct MeshData {
    mat4 transform;
};

//struct MaterialData {
//    vec4 ambient_color;
//};

layout (set = 0, binding = 0) uniform GlobalSceneData_UB {
    GlobalSceneData global_scene_data;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer{
	Vertex vertices[];
};

layout(buffer_reference, std430) readonly buffer MeshBuffer{
	MeshData mesh_data[];
};

//layout(buffer_reference, std430) readonly buffer MaterialBuffer{
//	MaterialData materials[];
//};

//push constants block
layout( push_constant ) uniform constants
{
	VertexBuffer  vertex_buffer;
	MeshBuffer    mesh_data_buffer;
	//MaterialData  material_buffer;
	uint          mesh_index;
	//uint          material_index;
} PushConstants;

void main()
{
	//load vertex data from device adress
	Vertex v = PushConstants.vertex_buffer.vertices[gl_VertexIndex];
	mat4 transform = PushConstants.mesh_data_buffer.mesh_data[PushConstants.mesh_index].transform;

	//output data
	gl_Position = global_scene_data.view_proj * transform * vec4(v.position, 1.0f);

	outColor = v.color.xyz;
	outUV.x = v.uv_x;
	outUV.y = v.uv_y;
}
