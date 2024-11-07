use futures::StreamExt;
use wasm_bindgen::prelude::*;
use web_sys::{console,Response,HtmlCanvasElement,WebGl2RenderingContext};
use futures::channel::mpsc;

#[derive(Debug, Clone)]
enum Shader{
    Vertex(String),
    Fragment(String),
}

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    //いつもの　window, document, bodyを取得
    let Some(window) = web_sys::window() else{
        console::log_1(&"Error: No window found".into());
        return Err(JsValue::NULL);
    };
    let Some(document) = window.document() else{
        console::log_1(&"Error: No document found".into());
        return Err(JsValue::NULL);
    };
    let Some(body) = document.body() else{
        console::log_1(&"Error: No body found".into());
        return Err(JsValue::NULL);
    };

    // canvas要素を作成してbodyに追加
    let Ok(canvas) = document.create_element("canvas") else{
        console::log_1(&"Error: could not create canvas element".into());
        return Err(JsValue::NULL);
    };

    body.append_child(&canvas).unwrap();

    // HtmlCanvasElementを取得
    let Ok(canvas_element) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() else{
        console::log_1(&"Error: could not convert to HtmlCanvasElement".into());
        return Err(JsValue::NULL);
    };

    canvas_element.set_width(500);
    canvas_element.set_height(500);
    
    console::log_1(&"set canvas success".into());

    let Ok(gl) = canvas_element.get_context("webgl2") else{
        console::log_1(&"Error: could not get webgl2 context".into());
        return Err(JsValue::NULL);
    };

    let Some(gl) = gl else{
        console::log_1(&"Error: could not get webgl2 context".into());
        return Err(JsValue::NULL);
    };

    let Ok(gl) = gl.dyn_into::<web_sys::WebGl2RenderingContext>() else{
        console::log_1(&"Error: could not convert to WebGl2RenderingContext".into());
        return Err(JsValue::NULL);
    };

    let(mut tx, mut rx) = mpsc::channel::<Shader>(32);

    let mut vertex_tx = tx.clone();

    wasm_bindgen_futures::spawn_local(async move{
        console::log_1(&"vertex shader read start".into());

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
        
        let value = Shader::Vertex(vertex_shader_source);
        loop{
            let Err(_) = vertex_tx.try_send(value.clone()) else{
                break;
            };
        }

        console::log_1(&"fragment shader read start".into());

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
        
        let value = Shader::Fragment(fragment_shader_source);
        loop{
            let Err(_) = tx.try_send(value.clone()) else{
                break;
            };
        }
    });

    wasm_bindgen_futures::spawn_local(async move{
        let mut vertex_shader_source:Option<String> = None;
        let mut is_vertex_received = false;
        let mut fragment_shader_source:Option<String> = None;
        let mut is_fragment_received = false;
        while let message = rx.next().await {
            let Some(message) = message else{
                console::log_1(&"message none".into());
                break;
            };
            match message{
                Shader::Vertex(source) => {
                    console::log_1(&"vertex shader".into());
                    console::log_1(&source.clone().into());
                    vertex_shader_source = Some(source);
                    is_vertex_received = true;
                },
                Shader::Fragment(source) => {
                    console::log_1(&"fragment shader".into());
                    console::log_1(&source.clone().into());
                    fragment_shader_source = Some(source);
                    is_fragment_received = true;
                },
            }

            if is_vertex_received && is_fragment_received{
                break;
            }
        };

        let vertex_shader = gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER).unwrap();
        let fragment_shader = gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER).unwrap();

        gl.shader_source(&vertex_shader, &vertex_shader_source.unwrap());
        gl.compile_shader(&vertex_shader);
        let vertex_status = gl
            .get_shader_parameter(&vertex_shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap();
        if !vertex_status {
            let log = gl.get_shader_info_log(&vertex_shader).unwrap();
            console::log_1(&log.into());
        }

        gl.shader_source(&fragment_shader, &fragment_shader_source.unwrap());
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

        let Some(program) = gl.create_program() else{
            console::log_1(&"program none value".into());
            panic!("program none value");
        };

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        let link_status = gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap();
        if !link_status{
            let log = gl.get_program_info_log(&program).unwrap();
            console::log_1(&log.into());
        }

        // プログラムを使用
        gl.use_program(Some(&program));
        console::log_1(&"use program success".into());

        const VERTEX_SIZE: i32 = 3;
        const COLOR_SIZE: i32 = 4;

        const FLOAT32_BYTES_PER_ELEMENT: i32 = 4;
        const STRIDE: i32 = (VERTEX_SIZE + COLOR_SIZE) * FLOAT32_BYTES_PER_ELEMENT;
        const POSITION_OFFSET: i32 = 0;
        const COLOR_OFFSET: i32 = VERTEX_SIZE * FLOAT32_BYTES_PER_ELEMENT;
        let vertices: [f32; 28] = [
            -0.5, 0.5, 0.0,
            1.0, 0.0, 0.0, 1.0,
            -0.5, -0.5, 0.0,
            0.0, 1.0, 0.0, 1.0,
            0.5, 0.5, 0.0,
            0.0, 0.0, 1.0, 1.0,
            0.5, -0.5, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let indices: [u16; 6] =[
            0,  1,  2,
            1,  3,  2,
        ];

        let interleaved_buffer = create_f32_buffer(WebGl2RenderingContext::ARRAY_BUFFER, &vertices, &gl).await.unwrap();
        let index_buffer = create_u16_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, &indices, &gl).await.unwrap();

        let vertex_attrib_location = gl.get_attrib_location(&program, "vertex_position");
        let color_attrib_location = gl.get_attrib_location(&program, "color");

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&interleaved_buffer));
        gl.enable_vertex_attrib_array(vertex_attrib_location as u32);
        gl.enable_vertex_attrib_array(color_attrib_location as u32);
        gl.vertex_attrib_pointer_with_i32(vertex_attrib_location as u32, VERTEX_SIZE, WebGl2RenderingContext::FLOAT, false, STRIDE, POSITION_OFFSET);
        gl.vertex_attrib_pointer_with_i32(color_attrib_location as u32, COLOR_SIZE, WebGl2RenderingContext::FLOAT, false, STRIDE, COLOR_OFFSET);

        let index_size = indices.len() as i32;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        gl.draw_elements_with_i32(WebGl2RenderingContext::TRIANGLES, index_size, WebGl2RenderingContext::UNSIGNED_SHORT, 0);
        gl.flush();
    });
    Ok(())
}

async fn create_f32_buffer(buffer_type: u32, typed_data_array: &[f32], gl: &WebGl2RenderingContext) -> Result<web_sys::WebGlBuffer, JsValue>{
    let buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(buffer_type, Some(&buffer));
    let array = js_sys::Float32Array::from(typed_data_array);
    gl.buffer_data_with_array_buffer_view(buffer_type, &array, WebGl2RenderingContext::STATIC_DRAW);

    // バッファのバインドを解除
    gl.bind_buffer(buffer_type, None);

    Ok(buffer)
}

async fn create_u16_buffer(buffer_type: u32, typed_data_array: &[u16], gl: &WebGl2RenderingContext) -> Result<web_sys::WebGlBuffer, JsValue>{
    let buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(buffer_type, Some(&buffer));
    let array = js_sys::Uint16Array::from(typed_data_array);
    gl.buffer_data_with_array_buffer_view(buffer_type, &array, WebGl2RenderingContext::STATIC_DRAW);

    // バッファのバインドを解除
    gl.bind_buffer(buffer_type, None);

    Ok(buffer)
}
