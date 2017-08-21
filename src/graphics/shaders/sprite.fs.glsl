#version 330 core

in vec2 texCoords;
in vec3 spriteColor;

out vec4 color;

uniform sampler2D image;

void main() {
    color = vec4(spriteColor, 1.0) * texture(image, texCoords);
}
