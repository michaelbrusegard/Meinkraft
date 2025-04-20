#version 410 core

in vec2 TexCoord;
in float LayerIndex;

out vec4 FragColor;

uniform sampler2DArray blockTexture;
uniform float lightLevel; // Uniform for overall light level

void main() {
    vec4 texColor = texture(blockTexture, vec3(TexCoord, LayerIndex));

    // Apply lighting - multiply RGB by lightLevel, keep alpha
    FragColor = vec4(texColor.rgb * lightLevel, texColor.a);

    // Discard fully transparent fragments (e.g., for glass edges)
    if (FragColor.a < 0.1) {
        discard;
    }
}
