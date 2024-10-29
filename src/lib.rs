use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement, WebGl2RenderingContext};
use std::path::Path;
use futures::channel::{mpsc, oneshot};

struct ShaderReadValue{
    vertex: String,
    fragment: String,
}

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

    console::log_1(&"canvas success".into());

    // WebGL2のコンテキストを取得
    let Some(gl_obj) = canvas_element.get_context("webgl2")? else{
        console::log_1(&"gl none value".into());
        return Ok(());
    };

    let gl = gl_obj.dyn_into::<WebGl2RenderingContext>()?;

    console::log_1(&"gl context success".into());

    let vertex_shader = gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER).unwrap();
    let fragment_shader = gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER).unwrap();

    let (error_tx, error_rx) = mpsc::channel::<String>(32);
    let (success_tx, success_rx) = oneshot::channel::<ShaderReadValue>();

    let read_glsls = wasm_bindgen_futures::spawn_local(async move{
        let vertex_shader_source = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str("../shader/vertex_shader.glsl")).await.unwrap().as_string().unwrap();
        let fragment_shader_source = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str("../shader/fragment_shader.glsl")).await.unwrap().as_string().unwrap();
        success_tx.send(ShaderReadValue{vertex: vertex_shader_source, fragment: fragment_shader_source});
    });


    let shader_read_val = success_rx.await.unwrap();
    let compile_shader = async move{
        gl.shader_source(&vertex_shader, &shader_read_val.vertex);
        gl.compile_shader(&vertex_shader);
        let vertex_status = gl.get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap();
        if !vertex_status {
            let log = gl.get_shader_info_log(&vertex_shader).unwrap();
            console::log_1(&log.into());
        }
        gl.shader_source(&fragment_shader, &shader_read_val.fragment);
        gl.compile_shader(&fragment_shader);
        let fragment_status = gl.get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap();
        if !fragment_status {
            let log = gl.get_shader_info_log(&fragment_shader).unwrap();
            console::log_1(&log.into());
        }
        console::log_1(&"shader compile success".into());

        let Some(program) = gl.create_program() else{
            console::log_1(&"program none value".into());
            panic!("program none value");
        };

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        
        // プログラムのリンクが成功したか確認
        let program_status = gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap();
        if !program_status {
            let log = gl.get_program_info_log(&program).unwrap();
            console::log_1(&log.into());
        }

        console::log_1(&"program link success".into());

        gl.use_program(Some(&program));

        let Some(vertex_buffer) = gl.create_buffer() else{
            console::log_1(&"vertex buffer none value".into());
            panic!("vertex buffer none value");
        };
        let Some(color_buffer) = gl.create_buffer() else{
            console::log_1(&"vertex buffer none value".into());
            panic!("color buffer none value");
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

        console::log_1(&"buffer data success".into());

        // 描画
        const VERTEX_NUMS: i32 = 6;
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, VERTEX_NUMS);

        gl.flush();
    };



    Ok(())
}
