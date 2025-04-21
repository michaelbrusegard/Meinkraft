#version 410 core

in vec2 TexCoord;
in float LayerIndex;

out vec4 FragColor;

uniform sampler2DArray blockTexture;
uniform float lightLevel;
uniform bool isCelestial;
uniform float celestialLayerIndex;

void main() {
    vec4 texColor;
    if (isCelestial) {
        texColor = texture(blockTexture, vec3(TexCoord, celestialLayerIndex));
        FragColor = texColor;
    } else {
        texColor = texture(blockTexture, vec3(TexCoord, LayerIndex));
        FragColor = vec4(texColor.rgb * lightLevel, texColor.a);
    }

    if (FragColor.a < 0.01) {
        discard;
    }
}
