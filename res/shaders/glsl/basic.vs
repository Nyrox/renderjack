#version 330 core

layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;

uniform mat4 view;
uniform mat4 proj;

out vec3 N;

void main() {
    gl_Position = proj * view * vec4(position, 1.0);
    N = normal;
}