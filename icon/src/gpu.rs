/// GPU-accelerated icon search using WebGPU compute shaders
/// 10-100x faster for large result sets
use wgpu::util::DeviceExt;

/// GPU search engine - offloads parallel search to GPU
pub struct GpuSearchEngine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl GpuSearchEngine {
    /// Initialize GPU engine (auto-detects best GPU)
    pub async fn new() -> Option<Self> {
        // Request GPU adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Icon Search GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .ok()?;

        // Load compute shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Icon Search Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/search.wgsl").into()),
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Icon Search Pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Some(Self {
            device,
            queue,
            pipeline,
        })
    }

    /// Search icons on GPU (parallel across all GPU cores)
    pub async fn search(
        &self,
        query: &str,
        icon_names: &[String],
    ) -> Result<Vec<u32>, anyhow::Error> {
        // Convert query to lowercase and then to u32 array
        let query_lower = query.to_lowercase();
        let query_u32: Vec<u32> = query_lower.as_bytes().iter().map(|&b| b as u32).collect();

        // Prepare GPU buffers
        let query_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Query Buffer"),
            contents: bytemuck::cast_slice(&query_u32),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Flatten icon names into single buffer as u32 (lowercase for case-insensitive matching)
        let mut icon_data_u32 = Vec::new();
        let mut offsets = Vec::new();
        for name in icon_names {
            offsets.push(icon_data_u32.len() as u32);
            let name_lower = name.to_lowercase();
            icon_data_u32.extend(name_lower.as_bytes().iter().map(|&b| b as u32));
        }

        let icon_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Icon Names Buffer"),
            contents: bytemuck::cast_slice(&icon_data_u32),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let offset_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offsets Buffer"),
            contents: bytemuck::cast_slice(&offsets),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Output buffer for results
        let result_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Result Buffer"),
            size: (icon_names.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Search Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: query_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: icon_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: offset_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        // Execute compute shader
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Search Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Search Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Dispatch workgroups (64 threads per workgroup)
            let workgroups = (icon_names.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Read results back
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (icon_names.len() * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &result_buffer,
            0,
            &staging_buffer,
            0,
            (icon_names.len() * 4) as u64,
        );

        self.queue.submit(Some(encoder.finish()));

        // Map buffer and read results
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = rx.receive().await {
            let data = buffer_slice.get_mapped_range();
            let results: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            staging_buffer.unmap();

            // Filter matching icons
            Ok(results
                .iter()
                .enumerate()
                .filter_map(|(idx, &score)| if score > 0 { Some(idx as u32) } else { None })
                .collect())
        } else {
            Err(anyhow::anyhow!("Failed to read GPU results"))
        }
    }

    /// Check if GPU is available
    pub async fn is_available() -> bool {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .is_some()
    }
}

/// Synchronous wrapper for GPU search
pub struct GpuSearchEngineSync {
    engine: Option<GpuSearchEngine>,
}

impl GpuSearchEngineSync {
    /// Initialize GPU engine (blocking)
    pub fn new() -> Self {
        let engine = pollster::block_on(GpuSearchEngine::new());
        Self { engine }
    }

    /// Search icons on GPU (blocking)
    pub fn search(&self, query: &str, icon_names: &[String]) -> Option<Vec<u32>> {
        let engine = self.engine.as_ref()?;
        pollster::block_on(engine.search(query, icon_names)).ok()
    }

    /// Check if GPU is available
    pub fn is_available(&self) -> bool {
        self.engine.is_some()
    }
}

impl Default for GpuSearchEngineSync {
    fn default() -> Self {
        Self::new()
    }
}
