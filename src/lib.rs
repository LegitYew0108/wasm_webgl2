use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement, WebGl2RenderingContext};
use std::fs;

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue>{
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // canvas要素を作成してbodyに追加
    let canvas= document.create_element("canvas")?;
    body.append_child(&canvas)?;

    let canvas_element = canvas.dyn_into::<HtmlCanvasElement>()?;
    canvas_element.set_width(500);
    canvas_element.set_height(500);

    // WebGL2のコンテキストを取得
    let Some(gl_obj) = canvas_element.get_context("webgl")? else{
        console::log_1(&"gl none value".into());
        return Ok(());
    };

    let gl = gl_obj.dyn_into::<WebGl2RenderingContext>()?;

    let vertex_shader = gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER).unwrap();

    let Ok(vertex_shader_source) = fs::read_to_string("shader/vertex.glsl") else{
        console::log_1(&"vertex shader source none value".into());
        return Ok(());
    };
    gl.shader_source(&vertex_shader, &vertex_shader_source);
    gl.compile_shader(&vertex_shader);

    let vertex_status = gl.get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap();
    if !vertex_status {
        let log = gl.get_shader_info_log(&vertex_shader).unwrap();
        console::log_1(&log.into());
        return Ok(());
    }

    let fragment_shader = gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER).unwrap();
    let Ok(fragment_shader_source) = fs::read_to_string("shader/fragment.glsl") else{
        console::log_1(&"fragment shader source none value".into());
        return Ok(());
    };
    gl.shader_source(&fragment_shader, &fragment_shader_source);
    gl.compile_shader(&fragment_shader);
    let fragment_status = gl.get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap();
    if !fragment_status {
        let log = gl.get_shader_info_log(&fragment_shader).unwrap();
        console::log_1(&log.into());
        return Ok(());
    }

    let Some(program) = gl.create_program() else{
        console::log_1(&"program none value".into());
        return Ok(());
    };

    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    // プログラムのリンクが成功したか確認
    let program_status = gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap();
    if !program_status {
        let log = gl.get_program_info_log(&program).unwrap();
        console::log_1(&log.into());
        return Ok(());
    }

    gl.use_program(Some(&program));

    let Some(vertex_buffer) = gl.create_buffer() else{
        console::log_1(&"vertex buffer none value".into());
        return Ok(());
    };
    let Some(color_buffer) = gl.create_buffer() else{
        console::log_1(&"vertex buffer none value".into());
        return Ok(());
    };

    let vertex_attrib_location = gl.get_attrib_location(&program, "vertex_position");
    let color_attrib_location = gl.get_attrib_location(&program, "color");

    const VERTEX_SIZE: i32 = 3;
    const COLOR_SIZE: i32 = 3;

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.enable_vertex_attrib_array(vertex_attrib_location as u32);

    gl.vertex_attrib_pointer_with_i32(vertex_attrib_location as u32, VERTEX_SIZE, WebGl2RenderingContext::FLOAT, false, 0, 0);

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    gl.enable_vertex_attrib_array(color_attrib_location as u32);

    gl.vertex_attrib_pointer_with_i32(color_attrib_location as u32, COLOR_SIZE, WebGl2RenderingContext::FLOAT, false, 0, 0);

    let vertices: [f32; 18] = [
            -0.5, 0.5,  0.0,
            -0.5, -0.5, 0.0,
            0.5,  0.5,  0.0,
            -0.5, -0.5, 0.0,
            0.5,  -0.5, 0.0,
            0.5,  0.5,  0.0
    ];

    let vertices = js_sys::Float32Array::from(vertices.as_ref());

    let colors: [f32; 24] = [
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 1.0
    ];

    let colors = js_sys::Float32Array::from(colors.as_ref());

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &vertices.into(), WebGl2RenderingContext::STATIC_DRAW);

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &colors.into(), WebGl2RenderingContext::STATIC_DRAW);

    // 描画
    const VERTEX_NUMS: i32 = 6;
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, VERTEX_NUMS);

    gl.flush();

    Ok(())
}
