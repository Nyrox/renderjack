#version 330 core

in vec3 normal;
out vec3 out_0;

vec3 __impl_main() {
	vec2 Vec2Test = vec2(1, 1);
	vec3 Vec3Test = vec3(0, 1, 3);
	vec4 Vec4Test = vec4(1, 4, 3.2, 3.1);
	mat2 Mat2Test = mat2(1, 0, 0, 1);
	mat2 Mat2VecTest = mat2(vec2(1, 0), vec2(0, 1));
	mat3 Mat3Test = mat3(1, 0, 0, 0, 1, 0, 0, 0, 1);
	mat3 Mat3VecTest = mat3(vec3(1, 0, 0), vec3(0, 1, 0), vec3(0, 0, 1));
	vec2 NormalizeTest = normalize(Vec2Test);
	vec3 L = normalize(vec3(-0.5, 1, -1));
	vec3 C = vec3(1, 0.5, 0.5);
	float cos_a = dot(L, normal);
	float ambient = 0.3;
	return cos_a * C + ambient * C;
}

void main() {
	vec3 rt = __impl_main();
	out_0 = rt;
}



