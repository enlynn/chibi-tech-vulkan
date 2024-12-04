use crate::util::ffi::call;

use vendor::vulkan::*;
use super::gpu_utils::*;
use super::gpu_device::Device;

use std::ptr;

pub struct GraphicsPipelineBuilder {
    shader_stages:           Vec<VkPipelineShaderStageCreateInfo>,
    input_assembly:          VkPipelineInputAssemblyStateCreateInfo,
    rasterizer:              VkPipelineRasterizationStateCreateInfo,
    color_blend_attachment:  VkPipelineColorBlendAttachmentState,
    multisampling:           VkPipelineMultisampleStateCreateInfo,
    pipeline_layout:         VkPipelineLayout,
    depth_stencil:           VkPipelineDepthStencilStateCreateInfo,
    render_info:             VkPipelineRenderingCreateInfo,
    color_attachment_format: VkFormat,
}

impl Default for GraphicsPipelineBuilder{
    fn default() -> Self {
        Self{
            shader_stages:           Vec::new(),
            input_assembly:          VkPipelineInputAssemblyStateCreateInfo::default(),
            rasterizer:              VkPipelineRasterizationStateCreateInfo::default(),
            color_blend_attachment:  VkPipelineColorBlendAttachmentState::default(),
            multisampling:           VkPipelineMultisampleStateCreateInfo::default(),
            pipeline_layout:         ptr::null_mut(),
            depth_stencil:           VkPipelineDepthStencilStateCreateInfo::default(),
            render_info:             VkPipelineRenderingCreateInfo::default(),
            color_attachment_format: VkFormat::default(),
        }
    }
}

impl GraphicsPipelineBuilder {
    pub fn new() -> Self {
        GraphicsPipelineBuilder::default()
    }

    pub fn clear(&mut self) {
        *self = GraphicsPipelineBuilder::default();
    }

    pub fn build(&mut self, device: &Device) -> VkPipeline {
        // Setup shader stage entry points
        let entry_point = std::ffi::CString::new("main").unwrap();
        for stage in &mut self.shader_stages {
            stage.pName = entry_point.as_ptr();
        }

        // make viewport state from our stored viewport and scissor.
        // at the moment we wont support multiple viewports or scissors
        let mut viewport_state = VkPipelineViewportStateCreateInfo::default();
        viewport_state.viewportCount = 1;
        viewport_state.scissorCount  = 1;

        // setup dummy color blending. We arent using transparent objects yet
        // the blending is just "no blend", but we do write to the color attachment
        let mut color_blending = VkPipelineColorBlendStateCreateInfo::default();
        color_blending.logicOp         = VK_LOGIC_OP_COPY;
        color_blending.attachmentCount = 1;
        color_blending.pAttachments    = &self.color_blend_attachment;

        // completely clear VertexInputStateCreateInfo, as we have no need for it (for now)
        let vertex_input_info = VkPipelineVertexInputStateCreateInfo::default();

        let render_info_ptr = &self.render_info as *const VkPipelineRenderingCreateInfo;

        let mut pipeline_ci = VkGraphicsPipelineCreateInfo::default();
        // connect the renderInfo to the pNext extension mechanism
        pipeline_ci.pNext               = render_info_ptr as *const _;
        pipeline_ci.stageCount          = self.shader_stages.len() as u32;
        pipeline_ci.pStages             = self.shader_stages.as_ptr();
        pipeline_ci.pVertexInputState   = &vertex_input_info;
        pipeline_ci.pInputAssemblyState = &self.input_assembly;
        pipeline_ci.pViewportState      = &viewport_state;
        pipeline_ci.pRasterizationState = &self.rasterizer;
        pipeline_ci.pMultisampleState   = &self.multisampling;
        pipeline_ci.pColorBlendState    = &color_blending;
        pipeline_ci.pDepthStencilState  = &self.depth_stencil;
        pipeline_ci.layout              = self.pipeline_layout;

        let state: [VkDynamicState; 2] = [ VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR ];

        let dynamic_ci = VkPipelineDynamicStateCreateInfo{
            sType:             VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext:             ptr::null(),
            flags:             0,
            dynamicStateCount: state.len() as u32,
            pDynamicStates:    state.as_ptr(),
        };

        pipeline_ci.pDynamicState = &dynamic_ci;

        device.create_graphics_pipeline(pipeline_ci)
    }

    pub fn set_pipeline_layout(&mut self, layout: VkPipelineLayout) -> &mut Self {
        self.pipeline_layout = layout;
        self
    }

    pub fn set_shaders(&mut self, vertex: VkShaderModule, fragment: VkShaderModule) -> &mut Self {
        self.shader_stages.clear();

        let vertex_stage_info = VkPipelineShaderStageCreateInfo{
            sType:               VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext:               ptr::null(),
            flags:               0,
            stage:               VK_SHADER_STAGE_VERTEX_BIT,
            module:              vertex,
            pName:               ptr::null(),
            //pName:               entry_point.as_ptr(),
            pSpecializationInfo: ptr::null(), //todo: specialization info
        };

        let fragment_stage_info = VkPipelineShaderStageCreateInfo{
            sType:               VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext:               ptr::null(),
            flags:               0,
            stage:               VK_SHADER_STAGE_FRAGMENT_BIT,
            module:              fragment,
            pName:               ptr::null(),
            //pName:               entry_point.as_ptr(),
            pSpecializationInfo: ptr::null(), //todo: specialization info
        };

        self.shader_stages.push(vertex_stage_info);
        self.shader_stages.push(fragment_stage_info);

        self
    }

    pub fn set_input_topology(&mut self, topology: VkPrimitiveTopology) -> &mut Self{
        self.input_assembly.topology = topology;
        // not going to use primitive restart so leave it on false
        self.input_assembly.primitiveRestartEnable = VK_FALSE;

        self
    }

    pub fn set_polygon_mode(&mut self, mode: VkPolygonMode) -> &mut Self {
        self.rasterizer.polygonMode = mode;
        self.rasterizer.lineWidth   = 1.0;

        self
    }

    pub fn set_cull_mode(&mut self, cull_mode: VkCullModeFlags, front_face: VkFrontFace) -> &mut Self{
        self.rasterizer.cullMode  = cull_mode;
        self.rasterizer.frontFace = front_face;

        self
    }

    pub fn set_multisampling_none(&mut self) -> &mut Self{
        self.multisampling.sampleShadingEnable   = VK_FALSE;
        // multisampling defaulted to no multisampling (1 sample per pixel)
        self.multisampling.rasterizationSamples  = VK_SAMPLE_COUNT_1_BIT;
        self.multisampling.minSampleShading      = 1.0;
        self.multisampling.pSampleMask           = ptr::null();
        // no alpha to coverage either
        self.multisampling.alphaToCoverageEnable = VK_FALSE;
        self.multisampling.alphaToOneEnable      = VK_FALSE;

        self
    }

    //
    // Blending
    //   outColor = srcColor * srcColorBlendFactor <op> dstColor * dstColorBlendFactor;
    //
    // VK_BLEND_FACTOR_ONE
    //   outColor = 1.0
    //
    // VK_BLEND_FACTOR_SRC_ALPHA
    //   outColor = srcColor.rgb * srcColor.a + dstColor.rgb * 1.0
    //
    // Alpha Blend
    //   outColor = srcColor.rgb * srcColor.a + dstColor.rgb * (1.0 - srcColor.a)
    //

    pub fn disable_blending(&mut self) -> &mut Self {
        // default write mask
        self.color_blend_attachment.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
        // no blending
        self.color_blend_attachment.blendEnable    = VK_FALSE;

        self
    }

    pub fn enabled_blending_additive(&mut self) -> &mut Self {
        self.color_blend_attachment.colorWriteMask      = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
        self.color_blend_attachment.blendEnable         = VK_TRUE;
        self.color_blend_attachment.srcColorBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA;
        self.color_blend_attachment.dstColorBlendFactor = VK_BLEND_FACTOR_ONE;
        self.color_blend_attachment.colorBlendOp        = VK_BLEND_OP_ADD;
        self.color_blend_attachment.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE;
        self.color_blend_attachment.dstAlphaBlendFactor = VK_BLEND_FACTOR_ZERO;
        self.color_blend_attachment.alphaBlendOp        = VK_BLEND_OP_ADD;

        self
    }

    pub fn enabled_blending_alphablend(&mut self) -> &mut Self {
        self.color_blend_attachment.colorWriteMask      = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
        self.color_blend_attachment.blendEnable         = VK_TRUE;
        self.color_blend_attachment.srcColorBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA;
        self.color_blend_attachment.dstColorBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
        self.color_blend_attachment.colorBlendOp        = VK_BLEND_OP_ADD;
        self.color_blend_attachment.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE;
        self.color_blend_attachment.dstAlphaBlendFactor = VK_BLEND_FACTOR_ZERO;
        self.color_blend_attachment.alphaBlendOp        = VK_BLEND_OP_ADD;

        self
    }

    pub fn set_color_attachment_format(&mut self, format: VkFormat) -> &mut Self {
        self.color_attachment_format = format;

        // connect the format to the renderInfo structure
        self.render_info.colorAttachmentCount    = 1;
        self.render_info.pColorAttachmentFormats = &self.color_attachment_format;

        self
    }

    pub fn set_depth_format(&mut self, format: VkFormat) -> &mut Self {
        self.render_info.depthAttachmentFormat = format;
        self
    }

    pub fn disable_depth_test(&mut self) -> &mut Self {
        self.depth_stencil.depthTestEnable       = VK_FALSE;
        self.depth_stencil.depthWriteEnable      = VK_FALSE;
        self.depth_stencil.depthCompareOp        = VK_COMPARE_OP_NEVER;
        self.depth_stencil.depthBoundsTestEnable = VK_FALSE;
        self.depth_stencil.stencilTestEnable     = VK_FALSE;
        self.depth_stencil.front                 = VkStencilOpState::default();
        self.depth_stencil.back                  = VkStencilOpState::default();
        self.depth_stencil.minDepthBounds        = 0.0;
        self.depth_stencil.maxDepthBounds        = 1.0;

        self
    }

    pub fn enable_depth_test(&mut self, depth_write_enabled: bool, op: VkCompareOp) -> &mut Self {
        self.depth_stencil.depthTestEnable       = VK_TRUE;
        self.depth_stencil.depthWriteEnable      = if depth_write_enabled { VK_TRUE } else { VK_FALSE };
        self.depth_stencil.depthCompareOp        = op;
        self.depth_stencil.depthBoundsTestEnable = VK_FALSE;
        self.depth_stencil.stencilTestEnable     = VK_FALSE;
        self.depth_stencil.front                 = VkStencilOpState::default();
        self.depth_stencil.back                  = VkStencilOpState::default();
        self.depth_stencil.minDepthBounds        = 0.0;
        self.depth_stencil.maxDepthBounds        = 1.0;

        self
    }
}
