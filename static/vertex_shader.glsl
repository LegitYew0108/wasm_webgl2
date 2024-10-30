#version 300 es 

in vec3 vertex_position;
in vec4 color;

out vec4 v_color;

void main(){
    // 頂点色をフラグメントシェーダにそのまま渡す
    v_color = color;

    // 頂点座標をそのまま使う
    gl_position = vec4(vertex_position, 1.0);
}
