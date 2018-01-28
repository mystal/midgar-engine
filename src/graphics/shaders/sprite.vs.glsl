#version 330 core

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords;
layout (location = 2) in vec4 color;

out vec2 texCoords;
out vec4 spriteColor;

// The combined projection/view matrix.
uniform mat4 projectionView;

void main() {
    texCoords = tex_coords;
    spriteColor = color;
    gl_Position = projectionView * vec4(pos, 0.0, 1.0);
}
