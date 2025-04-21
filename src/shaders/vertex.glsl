#version 410 core

layout(location = 0) in vec3 vertexPosition;
layout(location = 1) in vec2 vertexTexCoord;
layout(location = 2) in float vertexLayerIndex;

out vec2 TexCoord;
out float LayerIndex;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform bool isCelestial;

void main() {
    gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(vertexPosition, 1.0);

    TexCoord = vertexTexCoord;

    if (!isCelestial) {
        LayerIndex = vertexLayerIndex;
    }
}
