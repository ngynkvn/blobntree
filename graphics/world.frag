#version 330 core
in vec2 TexCoords;
out vec4 color;

uniform sampler2D image;
uniform ivec2 sprite_pos;
uniform ivec2 sprite_dim;

void main()
{    
    vec2 offset = TexCoords * sprite_dim;
    vec2 coords = sprite_pos + offset;
    ivec2 pos = ivec2(coords.x, coords.y);
    color = texelFetch(image, pos, 0);
}  