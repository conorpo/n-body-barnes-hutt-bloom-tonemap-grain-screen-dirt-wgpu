use wgpu::*;
use bytemuck;

pub struct Timestamps {
    pub query_set: QuerySet,
    buffer: Buffer,
    mapped_buffer: Buffer,
    pub last_frame_times: Vec<u64>
}

const TIMESTAMP_QUERY_COUNT: u32 = 4;

impl Timestamps {
    pub fn new(device: &Device) -> Self {
        // Start, After Render, After Bloom, After Presenting
        let timestamps = device.create_query_set(&QuerySetDescriptor { 
            label: Some("Timestamp QuerySet"), 
            ty: QueryType::Timestamp, 
            count: TIMESTAMP_QUERY_COUNT
        });
        let timestamps_buffer = device.create_buffer(&BufferDescriptor { 
            label: Some("Timestamp Buffer"), 
            size: (8 * TIMESTAMP_QUERY_COUNT as u64),
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC, 
            mapped_at_creation: false
        });
        let timestamps_mapped_buffer = device.create_buffer(&BufferDescriptor { 
            label: Some("Timestamp Mapped Buffer"), 
            size: (8 * TIMESTAMP_QUERY_COUNT as u64), 
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ, 
            mapped_at_creation: false
        });
        Self {
            query_set: timestamps,
            buffer: timestamps_buffer,
            mapped_buffer: timestamps_mapped_buffer,
            last_frame_times: vec![0;5]
        }
    }

    pub fn resolve(&mut self, encoder: &mut CommandEncoder) {
        encoder.resolve_query_set(&self.query_set, 0..TIMESTAMP_QUERY_COUNT, &self.buffer, 0);
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &self.mapped_buffer, 0, (8 * TIMESTAMP_QUERY_COUNT as u64));
    }

    pub fn update_times(&mut self, device: &mut Device) {
        self.mapped_buffer.slice(0..).map_async(MapMode::Read, |_| ());
        device.poll(MaintainBase::Wait).panic_on_timeout();

        self.last_frame_times = {
            let timestamps_mapped_buffer_view = self.mapped_buffer.slice(..(8 * TIMESTAMP_QUERY_COUNT as u64)).get_mapped_range();
            bytemuck::cast_slice(&timestamps_mapped_buffer_view).to_vec()
        };

        self.mapped_buffer.unmap();
    }


}