#version 410 core

layout(location = 0) in vec3 vertexPosition;
layout(location = 1) in vec2 vertexTexCoord;
layout(location = 2) in float vertexLayerIndex;
layout(location = 3) in vec3 vertexNormal;

out vec2 TexCoord;
out float LayerIndex;
out vec3 WorldNormal;
out vec3 WorldPos;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform bool isCelestial;

void main() {
    vec4 worldPosition4 = modelMatrix * vec4(vertexPosition, 1.0);
    gl_Position = projectionMatrix * viewMatrix * worldPosition4;

    TexCoord = vertexTexCoord;
    LayerIndex = vertexLayerIndex;

    WorldNormal = normalize(mat3(transpose(inverse(modelMatrix))) * vertexNormal);
    WorldPos = worldPosition4.xyz;
}
