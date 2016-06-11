#version 330 core

out vec4 color;

uniform sampler2D image;
uniform vec3 shapeColor;

void main() {
    color = vec4(shapeColor, 1.0);
}
