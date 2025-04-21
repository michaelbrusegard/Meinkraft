#version 410 core

in float starSeed;

out vec4 FragColor;

uniform float time;
uniform float nightFactor;

void main() {
    float seed = starSeed;

    float twinkle = 0.6 + 0.4 * sin(time * (0.5 + seed * 1.5) + seed * 10.0);
    twinkle *= 0.8 + 0.2 * cos(time * (0.2 + seed * 0.8) + seed * 5.0);
    twinkle = pow(twinkle, 2.0);
    twinkle = clamp(twinkle, 0.2, 1.0);

    vec3 starColor = vec3(1.0) * twinkle;

    float temporalAlpha = smoothstep(0.75, 0.95, nightFactor);

    float finalAlpha = temporalAlpha;

    FragColor = vec4(starColor, finalAlpha);

    if (FragColor.a < 0.01) {
        discard;
    }
}
