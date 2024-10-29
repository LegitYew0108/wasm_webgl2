#version 300 es 

// floatの精度を指定
precision highp float;

// 頂点シェーダから受け取る頂点色
in vec4 v_color;

// 出力する色
out vec4 fragment_color;

void main(){
    // 頂点色をそのまま使う
    fragment_color = v_color;
}
