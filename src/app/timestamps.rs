use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::{Arc, Mutex}};

use wgpu::*;
use bytemuck;

pub struct Timestamps {
    pub query_set: QuerySet,
    buffer: Buffer,
    pub last_frame_times: Arc<Mutex<[u64; TIMESTAMP_QUERY_COUNT as usize]>>,

    unmapped_ring: Arc<Mutex<VecDeque<Buffer>>>,
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
        let timestamps_mapped_buffer = device.create_buffer(&Timestamps::get_ring_buffer_descriptor());
        
        Self {
            query_set: timestamps,
            buffer: timestamps_buffer,
            unmapped_ring: Arc::new(Mutex::new(vec![timestamps_mapped_buffer].into())),
            last_frame_times: Arc::new(Mutex::new([0;4]))
        }
            
    }

    fn get_ring_buffer_descriptor() -> BufferDescriptor<'static> {
        BufferDescriptor {
            label: Some("Timestamp Mapped Buffer"), 
            size: (8 * TIMESTAMP_QUERY_COUNT as u64), 
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ, 
            mapped_at_creation: false
        }
    }


    pub fn resolve(&mut self, encoder: &mut CommandEncoder, device: &Device) {
        encoder.resolve_query_set(&self.query_set, 0..TIMESTAMP_QUERY_COUNT, &self.buffer, 0);

        let mut unmapped_ring = self.unmapped_ring.lock().unwrap();
        
        if unmapped_ring.is_empty() {
            unmapped_ring.push_back(device.create_buffer(&Timestamps::get_ring_buffer_descriptor()));
        };
        encoder.copy_buffer_to_buffer(&self.buffer, 0, unmapped_ring.front().unwrap(), 0, 8 * TIMESTAMP_QUERY_COUNT as u64);
    }

    pub fn update_times(&mut self, device: &mut Device) {
        //Map the buffer we just wrote to
        let buffer_to_map = Arc::<Buffer>::new(self.unmapped_ring.lock().unwrap().pop_front().unwrap());
        let last_frame_times = self.last_frame_times.clone();
        let mut unmapped_ring = self.unmapped_ring.clone();
        
        buffer_to_map.clone().slice(..(8 * TIMESTAMP_QUERY_COUNT as u64)).map_async(MapMode::Read, move |res| {
            let buffer_view = buffer_to_map.slice(..(8 * TIMESTAMP_QUERY_COUNT as u64)).get_mapped_range();
            (*last_frame_times.lock().unwrap()).copy_from_slice(bytemuck::cast_slice(&buffer_view));
            drop(buffer_view);

            buffer_to_map.unmap();
            unmapped_ring.lock().unwrap().push_back(Arc::into_inner(buffer_to_map).unwrap()); //change this shit

        });
        //self.mapped_ring.push_back(buffer_to_map);
    }
}