#version 410 core

in vec2 TexCoord;
in float LayerIndex;
in vec3 WorldNormal;
in vec3 WorldPos;

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

        vec3 lighting = finalAmbient + diffuse;

        finalColor = texColor.rgb * lighting + specular;
    }

    FragColor = vec4(finalColor, texColor.a);

    if (FragColor.a < 0.01) {
        discard;
    }
}
