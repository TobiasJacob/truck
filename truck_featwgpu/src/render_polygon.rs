use crate::*;
use polymesh::*;
use std::collections::HashMap;

impl ExpandedPolygon {
    pub fn buffers(&self, device: &Device) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsage::VERTEX,
                label: None,
            }),
            (self.vertices.len() * std::mem::size_of::<AttrVertex>()) as u64,
        );
        let index_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.indices),
                usage: BufferUsage::INDEX,
                label: None,
            }),
            self.indices.len() as u64,
        );
        (vertex_buffer, index_buffer)
    }
}

impl Default for ColorConfig {
    #[inline(always)]
    fn default() -> ColorConfig {
        ColorConfig {
            ambient: Vector4::new(1.0, 1.0, 1.0, 1.0),
            diffuse: Vector4::new(1.0, 1.0, 1.0, 1.0),
            specular: Vector4::new(1.0, 1.0, 1.0, 1.0),
            reflect_ratio: Vector3::new(0.2, 0.6, 0.2),
        }
    }
}

impl Rendered for PolygonInstance {
    fn vertex_buffer(&self, _: &Device) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        (Arc::clone(&self.polygon.0), Some(Arc::clone(&self.polygon.1)))
    }
    fn bind_group_layout(&self, device: &Device) -> Arc<BindGroupLayout> {
        let layout = if self.texture.is_some() {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    // matrix
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // color
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // texture
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Uint,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // sampler
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: None,
            })
        } else {
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    // matrix
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // color
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            })
        };
        Arc::new(layout)
    }
    fn bind_group(&self, device: &Device, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let matrix_data: [[f32; 4]; 4] = self.matrix.cast::<f32>().unwrap().into();
        let matrix_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&matrix_data),
                usage: BufferUsage::UNIFORM,
                label: None,
            }),
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        );
        let rr = self.color.reflect_ratio;
        let color_data: [[f32; 4]; 4] = [
            self.color.ambient.cast::<f32>().unwrap().into(),
            self.color.diffuse.cast::<f32>().unwrap().into(),
            self.color.specular.cast::<f32>().unwrap().into(),
            [rr[0] as f32, rr[1] as f32, rr[2] as f32, 0.0],
        ];
        let color_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&color_data),
                usage: BufferUsage::UNIFORM,
                label: None,
            }),
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        );
        let bind_group = buffer_handler::create_bind_group(device, layout, &[matrix_buffer, color_buffer]);
        Arc::new(bind_group)
    }
    fn pipeline(&self, device: &Device, sc_desc: &SwapChainDescriptor, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let vertex_module = device.create_shader_module(include_spirv!("polygon.vert.spv"));
        let fragment_module = device.create_shader_module(include_spirv!("polygon.frag.spv"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 2 * 4 + 3 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }
}

impl PolygonInstance {
    pub fn new<T: Into<ExpandedPolygon>>(polygon: T, device: &Device) -> PolygonInstance {
        let (vb, ib) = polygon.into().buffers(device);
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            matrix: Matrix4::identity(),
            color: Default::default(),
            texture: None,
        }
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut ExpandedPolygon,
    vertex_map: &mut HashMap<[usize; 3], u32>,
)
{
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).cast().unwrap().into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).cast().unwrap().into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).cast().unwrap().into(),
            };
            let wgpuvertex = AttrVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(wgpuvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl Default for ExpandedPolygon {
    fn default() -> ExpandedPolygon {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl From<&PolygonMesh> for ExpandedPolygon {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let mut vertex_map = HashMap::<[usize; 3], u32>::new();
        for tri in &polymesh.tri_faces {
            signup_vertex(polymesh, &tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in &polymesh.quad_faces {
            signup_vertex(polymesh, &quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in &polymesh.other_faces {
            for i in 2..face.len() {
                signup_vertex(polymesh, &face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<PolygonMesh> for ExpandedPolygon {
    #[inline(always)]
    fn from(polymesh: PolygonMesh) -> ExpandedPolygon { (&polymesh).into() }
}

impl From<StructuredMesh> for ExpandedPolygon {
    #[inline(always)]
    fn from(mesh: StructuredMesh) -> ExpandedPolygon { mesh.destruct().into() }
}

impl From<MeshHandler> for ExpandedPolygon {
    #[inline(always)]
    fn from(mesh_handler: MeshHandler) -> ExpandedPolygon {
        Into::<PolygonMesh>::into(mesh_handler).into()
    }
}