use super::super::context::Context;
use ash::{Device, prelude::VkResult, vk};
use std::rc::Rc;

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shader/basic.vert.spv"
));
const FRAGMENT_SHADER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/shader/basic.frag.spv"
));

pub struct Pipeline {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    ctx: Rc<Context>,
}

impl Pipeline {
    pub fn new(ctx: Rc<Context>, render_pass: vk::RenderPass, area: vk::Rect2D) -> VkResult<Self> {
        let pipeline_layout = create_pipeline_layout(&ctx.device, &[])?;
        let pipeline = create_pipeline(&ctx.device, render_pass, pipeline_layout, area)?;
        Ok(Self {
            pipeline_layout,
            pipeline,
            ctx,
        })
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.destroy_pipeline(self.pipeline, None);
            self.ctx
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn create_pipeline_layout(
    device: &Device,
    set_layouts: &[vk::DescriptorSetLayout],
) -> VkResult<vk::PipelineLayout> {
    let ci = vk::PipelineLayoutCreateInfo::default().set_layouts(set_layouts);
    unsafe { device.create_pipeline_layout(&ci, None) }
}

fn create_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    area: vk::Rect2D,
) -> VkResult<vk::Pipeline> {
    // shader stages
    let vs = create_shader_module(device, VERTEX_SHADER)?;
    let fs = create_shader_module(device, FRAGMENT_SHADER)?;
    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .module(vs)
            .name(c"main")
            .stage(vk::ShaderStageFlags::VERTEX),
        vk::PipelineShaderStageCreateInfo::default()
            .module(fs)
            .name(c"main")
            .stage(vk::ShaderStageFlags::FRAGMENT),
    ];

    // vertex input
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();

    // input assembly
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_STRIP);

    // rasterization state
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);

    // color blend
    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blend_state =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

    // viewport state
    let viewports = [vk::Viewport {
        x: area.offset.x as f32,
        y: area.offset.y as f32,
        width: area.extent.width as f32,
        height: area.extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [area];
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(&viewports)
        .scissors(&scissors);

    // multisample state
    let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    // pipeline
    let ci = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);
    let pipelines =
        unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[ci], None) };
    let pipeline = match pipelines {
        Ok(pipelines) if !pipelines.is_empty() => Ok(pipelines[0]),
        Ok(_) => Err(vk::Result::ERROR_UNKNOWN),
        Err((_, e)) => Err(e),
    };

    // シェーダモジュールは不要なので破棄
    unsafe { device.destroy_shader_module(fs, None) };
    unsafe { device.destroy_shader_module(vs, None) };

    pipeline
}

fn create_shader_module(device: &Device, spv_bin: &[u8]) -> VkResult<vk::ShaderModule> {
    let code = spv_bin
        .chunks_exact(4)
        .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    let ci = ash::vk::ShaderModuleCreateInfo::default().code(&code);
    unsafe { device.create_shader_module(&ci, None) }
}
