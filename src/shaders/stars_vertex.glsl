#version 410 core

layout(location = 0) in vec3 starDirection;

out float starSeed;

uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform float starDistance;

float hash(vec3 p) {
    return fract(sin(dot(p, vec3(12.9898, 78.233, 54.321))) * 43758.5453);
}

void main() {
    starSeed = hash(starDirection);

    vec4 viewPosition = vec4(starDirection * starDistance, 1.0);
    gl_Position = projectionMatrix * viewMatrix * viewPosition;

    gl_PointSize = 2.0;
}
