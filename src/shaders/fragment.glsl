#version 410 core

in vec2 TexCoord;
in float LayerIndex;

out vec4 FragColor;

uniform sampler2DArray blockTexture;

void main() {
    FragColor = texture(blockTexture, vec3(TexCoord, LayerIndex));

    if (FragColor.a < 0.1) {
        discard;
    }
}
