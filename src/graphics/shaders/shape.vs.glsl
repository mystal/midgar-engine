#version 330 core

layout (location = 0) in vec2 pos;
layout (location = 1) in vec4 color;

out vec4 shapeColor;

// The combined projection/view matrix.
uniform mat4 projectionView;

void main() {
    shapeColor = color;
    gl_Position = projectionView * vec4(pos, 0.0, 1.0);
}
