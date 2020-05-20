#version 330 core


in vec3 N;

out vec4 fb_col;


void main() {

    vec3 L = vec3(-0.5, 1.0, -1.0);
    float cosa = max(dot(L, N), 0.0);

    vec3 color = vec3(1.0, 0.5, 0.5);
    vec3 final = color * cosa;


    fb_col = vec4(final, 1.0);
}