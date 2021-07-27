#version 330 core
in vec2 pos; // <vec2 position, vec2 texCoords>
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 projection;

void main()
{
    TexCoords = pos;
    gl_Position = projection * model * vec4(pos, 0.0, 1.0);
}