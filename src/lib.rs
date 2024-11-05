use futures::channel::oneshot;
use wasm_bindgen::prelude::*;
use web_sys::{console, HtmlCanvasElement, WebGl2RenderingContext,Response};

#[derive(Debug, Clone)]
struct ShaderReadValue {
    vertex: String,
    fragment: String,
}

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // canvas要素を作成してbodyに追加
    let canvas = document.create_element("canvas")?;
    body.append_child(&canvas)?;

    let canvas_element = canvas.dyn_into::<HtmlCanvasElement>()?;
    canvas_element.set_width(500);
    canvas_element.set_height(500);

    console::log_1(&"canvas success".into());

    // WebGL2のコンテキストを取得
    let Some(gl_obj) = canvas_element.get_context("webgl2")? else {
        console::log_1(&"gl none value".into());
        return Ok(());
    };

    let gl = gl_obj.dyn_into::<WebGl2RenderingContext>()?;

    console::log_1(&"gl context success".into());

    let vertex_shader = gl
        .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
        .unwrap();
    let fragment_shader = gl
        .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
        .unwrap();

    let (success_tx, success_rx) = oneshot::channel::<ShaderReadValue>();

    wasm_bindgen_futures::spawn_local(async move {
        console::log_1(&"shader read start".into());
        // vertex shaderを読み出す
        let Ok(vertex_shader_source) = wasm_bindgen_futures::JsFuture::from(
            window.fetch_with_str("../shader/vertex_shader.glsl"),
        )
        .await else{
            console::log_1(&"shader read failed".into());
            panic!("shader read failed");
        };
        let Ok(vertex_shader_source) = vertex_shader_source
        .dyn_into::<Response>() else{
            console::log_1(&"dynamic cast to Response failed".into());
            panic!("shader read failed");
        };
        let Ok(vertex_shader_source) = vertex_shader_source.text() else{
            console::log_1(&"could not change to text".into());
            panic!("shader read failed");
        };
        let Ok(vertex_shader_source) = wasm_bindgen_futures::JsFuture::from(vertex_shader_source).await else{
            console::log_1(&"promise failed".into());
            panic!("shader read failed");
        };

        let Some(vertex_shader_source) = vertex_shader_source.as_string() else{
            console::log_1(&"shader source none".into());
            panic!("shader read failed");
        };

        // fragment shaderを読み出す
        let Ok(fragment_shader_source) = wasm_bindgen_futures::JsFuture::from(
            window.fetch_with_str("../shader/fragment_shader.glsl"),
        )
        .await else{
            console::log_1(&"shader read failed".into());
            panic!("shader read failed");
        };
        let Ok(fragment_shader_source) = fragment_shader_source
        .dyn_into::<Response>() else{
            console::log_1(&"dynamic cast to Response failed".into());
            panic!("shader read failed");
        };
        let Ok(fragment_shader_source) = fragment_shader_source.text() else{
            console::log_1(&"could not change to text".into());
            panic!("shader read failed");
        };
        let Ok(fragment_shader_source) = wasm_bindgen_futures::JsFuture::from(fragment_shader_source).await else{
            console::log_1(&"promise failed".into());
            panic!("shader read failed");
        };

        let Some(fragment_shader_source) = fragment_shader_source.as_string() else{
            console::log_1(&"shader source none".into());
            panic!("shader read failed");
        };
        let _ = success_tx.send(ShaderReadValue {
            vertex: vertex_shader_source,
            fragment: fragment_shader_source,
        });
    });

    wasm_bindgen_futures::spawn_local(async move {
        let shader_sources = success_rx.await;
        gl.shader_source(&vertex_shader, &shader_sources.clone().unwrap().vertex);
        gl.compile_shader(&vertex_shader);
        let vertex_status = gl
            .get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap();
        if !vertex_status {
            let log = gl.get_shader_info_log(&vertex_shader).unwrap();
            console::log_1(&log.into());
        }

        gl.shader_source(&fragment_shader, &shader_sources.clone().unwrap().fragment);
        gl.compile_shader(&fragment_shader);
        let fragment_status = gl
            .get_shader_parameter(&fragment_shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap();
        if !fragment_status {
            let log = gl.get_shader_info_log(&fragment_shader).unwrap();
            console::log_1(&log.into());
        }
        console::log_1(&"shader compile success".into());

        let Some(program) = gl.create_program() else {
            console::log_1(&"program none value".into());
            panic!("program none value");
        };

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        // プログラムのリンクが成功したか確認
        let program_status = gl
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap();
        if !program_status {
            let log = gl.get_program_info_log(&program).unwrap();
            console::log_1(&log.into());
        }

        console::log_1(&"program link success".into());

        gl.use_program(Some(&program));

        let Some(vertex_buffer) = gl.create_buffer() else {
            console::log_1(&"vertex buffer none value".into());
            panic!("vertex buffer none value");
        };

        let vertex_attrib_location = gl.get_attrib_location(&program, "vertex_position");
        let color_attrib_location = gl.get_attrib_location(&program, "color");

        const VERTEX_SIZE: i32 = 3;
        const COLOR_SIZE: i32 = 4;

        const FLOAT32_BYTES_PER_ELEMENT: i32 = 4;
        const STRIDE: i32 = (VERTEX_SIZE + COLOR_SIZE) * FLOAT32_BYTES_PER_ELEMENT;
        const POSITION_OFFSET: i32 = 0;
        const COLOR_OFFSET: i32 = VERTEX_SIZE * FLOAT32_BYTES_PER_ELEMENT;

        //バッファをバインド
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        //in変数の有効化(glsl)
        gl.enable_vertex_attrib_array(vertex_attrib_location as u32);
        gl.enable_vertex_attrib_array(color_attrib_location as u32);

        gl.vertex_attrib_pointer_with_i32(
            vertex_attrib_location as u32,
            VERTEX_SIZE,
            WebGl2RenderingContext::FLOAT,
            false,
            STRIDE,
            POSITION_OFFSET,
        );
        gl.vertex_attrib_pointer_with_i32(
            color_attrib_location as u32,
            COLOR_SIZE,
            WebGl2RenderingContext::FLOAT,
            false,
            STRIDE,
            COLOR_OFFSET,
        );

        let vertices: [f32; 42] = [
            -0.5, 0.5, 0.0,
            1.0, 0.0, 0.0, 1.0,
            -0.5, -0.5, 0.0,
            0.0, 1.0, 0.0, 1.0,
            0.5, 0.5, 0.0,
            0.0, 0.0, 1.0, 1.0,
            -0.5, -0.5, 0.0,
            0.0, 1.0, 0.0, 1.0,
            0.5, -0.5, 0.0,
            0.0, 0.0, 0.0, 1.0,
            0.5, 0.5, 0.0,
            0.0, 0.0, 1.0, 1.0,
        ];

        let vertices = js_sys::Float32Array::from(vertices.as_ref());

        //バインドしてデータを転送
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices.into(),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        console::log_1(&"buffer data success".into());

        // 描画
        const VERTEX_NUMS: i32 = 6;
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, VERTEX_NUMS);

        gl.flush();
    });
    Ok(())
}
