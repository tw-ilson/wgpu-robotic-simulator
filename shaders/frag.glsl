#version 440 core
in vec3 theColor;
out vec4 color;
void main()
{
  color = vec4(color.r,color.g,color.b,1.0);
}

