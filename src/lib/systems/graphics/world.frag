#version 330 core
in vec2 TexCoords;
out vec4 color;

uniform sampler2DArray image;
uniform int index;
uniform vec2 sprite_dim;

void main()
{    
    // ivec2 pos = ivec2(coords.x, coords.y);
    // color = texelFetch(image, pos, 0);
    color = texture(image, vec3(TexCoords, index));
}  