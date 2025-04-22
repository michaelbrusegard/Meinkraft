#version 410 core

in vec2 TexCoord;
in float LayerIndex;
in vec3 WorldNormal;
in vec3 WorldPos;
in vec4 FragPosLightSpace;

out vec4 FragColor;

uniform sampler2DArray blockTexture;
uniform vec3 lightDirection;
uniform vec3 ambientColor;
uniform vec3 lightColor;
uniform float minAmbientContribution;
uniform bool isCelestial;
uniform float celestialLayerIndex;
uniform vec3 cameraPosition;
uniform float shininess;
uniform sampler2D shadowMap;

float calculateShadow(vec3 norm, vec3 lightDir, vec4 fragPosLightSpace) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;

    projCoords = projCoords * 0.5 + 0.5;

    float closestDepth = texture(shadowMap, projCoords.xy).r;

    float currentDepth = projCoords.z;

    float bias = 0.009;

    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;

    if (projCoords.z > 1.0)
        shadow = 0.0;

    return 1.0 - shadow;
}

void main() {
    vec4 texColor;
    vec3 finalColor;

    vec3 norm = normalize(WorldNormal);
    vec3 lightDir = normalize(lightDirection);
    vec3 viewDir = normalize(cameraPosition - WorldPos);

    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    vec3 specular = lightColor * spec;

    if (isCelestial) {
        texColor = texture(blockTexture, vec3(TexCoord, celestialLayerIndex));
        finalColor = texColor.rgb + specular;
    } else {
        texColor = texture(blockTexture, vec3(TexCoord, LayerIndex));

        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = lightColor * diff;

        vec3 finalAmbient = max(ambientColor, vec3(minAmbientContribution));

        float shadow = calculateShadow(norm, lightDir, FragPosLightSpace); // Re-enabled call

        vec3 lighting = finalAmbient + (diffuse + specular) * shadow;

        finalColor = texColor.rgb * lighting;
    }

    FragColor = vec4(finalColor, texColor.a);

    if (FragColor.a < 0.01) {
        discard;
    }
}
