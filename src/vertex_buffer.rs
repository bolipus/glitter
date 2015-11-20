use std::marker::PhantomData;
use std::collections::{HashMap, HashSet};
use context::Context;
use framebuffer::FramebufferBinding;
use program::ProgramAttrib;
use vertex_data::{VertexData, VertexBytes, VertexAttribute};
use index_data::{IndexData, IndexDatum};
use buffer::{Buffer, BufferBinding,
             ArrayBufferBinding, ElementArrayBufferBinding};
use types::DrawingMode;

#[derive(Debug)]
pub enum AttribAddError {
    DuplicateAttrib(String)
}

#[derive(Debug)]
pub struct AttribError {
    missing_attribs: Vec<String>,
    unknown_attribs: Vec<String>
}

pub struct AttribBinder {
    attribs: HashMap<String, ProgramAttrib>
}

impl AttribBinder {
    pub fn new() -> Self {
        AttribBinder {
            attribs: HashMap::new()
        }
    }

    pub fn add(&mut self, name: &str, attrib: ProgramAttrib)
        -> Result<(), AttribAddError>
    {
        match self.attribs.insert(name.into(), attrib) {
            None => Ok(()),
            Some(_) => Err(AttribAddError::DuplicateAttrib(name.into()))
        }
    }

    fn for_each<T, F>(&self, mut f: F) -> Result<(), AttribError>
        where T: VertexData, F: FnMut(VertexAttribute, ProgramAttrib)
    {
        // TODO: Avoid heap allocations
        // TODO: Avoid redundant calls to T::visit_attributes
        let mut attribs =
            HashMap::<String, (VertexAttribute, ProgramAttrib)>::new();
        let mut missing = Vec::<String>::new();

        T::visit_attributes(|vertex_attrib| {
            match self.attribs.get(&vertex_attrib.name) {
                Some(program_attrib) => {
                    let pair = (vertex_attrib.clone(), *program_attrib);
                    attribs.insert(vertex_attrib.name, pair);
                },
                None => {
                    missing.push(vertex_attrib.name);
                }
            }
        });

        let unknown: Vec<_> = {
            let expected: HashSet<_> = self.attribs.keys().collect();
            let actual: HashSet<_> = attribs.keys().collect();
            expected.difference(&actual).cloned().cloned().collect()
        };

        if missing.is_empty() && unknown.is_empty() {
            for (_, (vertex_attrib, program_attrib)) in attribs.into_iter() {
                f(vertex_attrib, program_attrib);
            }
            Ok(())
        }
        else {
            Err(AttribError {
                missing_attribs: missing,
                unknown_attribs: unknown
            })
        }
    }


    pub fn enable<T: VertexData>(&self, gl: &Context)
        -> Result<(), AttribError>
    {
        self.for_each::<T, _>(|_, program_attrib| {
            gl.enable_vertex_attrib_array(program_attrib)
        })
    }

    pub fn bind<T: VertexData>(&self, gl_buffer: &ArrayBufferBinding)
        -> Result<(), AttribError>
    {
        self.for_each::<T, _>(|vertex_attrib, program_attrib| {
            unsafe {
                gl_buffer.vertex_attrib_pointer(
                    program_attrib,
                    vertex_attrib.ty.components,
                    vertex_attrib.ty.data,
                    vertex_attrib.ty.normalize,
                    vertex_attrib.stride,
                    vertex_attrib.offset
                );
            }
        })
    }
}



pub struct VertexBuffer<T: VertexData> {
    attrib_binder: Option<AttribBinder>,
    buffer: Buffer,
    count: usize,
    phantom: PhantomData<*const T>
}

impl<T: VertexData> VertexBuffer<T> {
    pub fn bind_attrib_pointers(&mut self, binder: AttribBinder) {
        self.attrib_binder = Some(binder);
    }

    pub fn bind(&self, gl_buffer: &ArrayBufferBinding) -> Result<(), ()> {
        match self.attrib_binder {
            Some(ref binder) => {
                let mut gl = unsafe { Context::current_context() };
                try!(binder.enable::<T>(&mut gl).or(Err(())));
                try!(binder.bind::<T>(gl_buffer).or(Err(())));
                Ok(())
            },
            None => { Err(()) }
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

pub struct VertexBufferBinding<'a, T: VertexData + 'a> {
    gl_buffer: ArrayBufferBinding<'a>,
    vbo: &'a mut VertexBuffer<T>
}

impl<'a, T: VertexData + 'a> VertexBufferBinding<'a, T> {
    pub fn new(gl_buffer: ArrayBufferBinding<'a>, vbo: &'a mut VertexBuffer<T>)
        -> Self
    {
        VertexBufferBinding {
            gl_buffer: gl_buffer,
            vbo: vbo
        }
    }

    pub fn buffer_data(&mut self, data: &[T], usage: super::BufferDataUsage)
        where [T]: VertexBytes
    {
        self.vbo.count = data.len();
        self.gl_buffer.buffer_bytes(data.vertex_bytes(), usage);
    }
}

impl<'a> FramebufferBinding<'a> {
    pub fn draw_arrays_range_vbo<T>(&mut self,
                                    gl_vbo: &VertexBufferBinding<T>,
                                    mode: DrawingMode,
                                    start: u32,
                                    length: usize)
        where T: VertexData
    {
        debug_assert!((start as usize) + length <= gl_vbo.vbo.count);

        unsafe {
            self.draw_arrays_range(&gl_vbo.gl_buffer, mode, start, length);
        }
    }

    pub fn draw_arrays_vbo<T>(&mut self,
                              gl_vbo: &VertexBufferBinding<T>,
                              mode: DrawingMode)
        where T: VertexData
    {
        unsafe {
            self.draw_arrays_range(&gl_vbo.gl_buffer,
                                   mode,
                                   0,
                                   gl_vbo.vbo.count);
        }
    }

    pub fn draw_n_elements_buffered_vbo<T, I>(&mut self,
                                              gl_vbo: &VertexBufferBinding<T>,
                                              gl_ibo: &IndexBufferBinding<I>,
                                              mode: DrawingMode,
                                              length: usize)
        where T: VertexData, I: IndexDatum
    {
        debug_assert!(length <= gl_ibo.ibo.count);

        unsafe {
            self.draw_n_elements_buffered(&gl_vbo.gl_buffer,
                                          &gl_ibo.gl_buffer,
                                          mode,
                                          length,
                                          I::index_datum_type());
        }
    }

    pub fn draw_elements_buffered_vbo<T, I>(&mut self,
                                            gl_vbo: &VertexBufferBinding<T>,
                                            gl_ibo: &IndexBufferBinding<I>,
                                            mode: DrawingMode)
        where T: VertexData, I: IndexDatum
    {
        unsafe {
            self.draw_n_elements_buffered(&gl_vbo.gl_buffer,
                                          &gl_ibo.gl_buffer,
                                          mode,
                                          gl_ibo.ibo.count,
                                          I::index_datum_type());
        }
    }

    pub fn draw_n_elements_vbo<T, I>(&mut self,
                                     gl_vbo: &VertexBufferBinding<'a, T>,
                                     mode: DrawingMode,
                                     count: usize,
                                     indices: &[I])
        where T: VertexData, I: IndexDatum, [I]: IndexData
    {
        unsafe {
            self.draw_n_elements(&gl_vbo.gl_buffer, mode, count, indices);
        }
    }

    pub fn draw_elements_vbo<T, I>(&mut self,
                                   gl_vbo: &VertexBufferBinding<'a, T>,
                                   mode: DrawingMode,
                                   indices: &[I])
        where T: VertexData, I: IndexDatum, [I]: IndexData
    {
        unsafe {
            self.draw_elements(&gl_vbo.gl_buffer, mode, indices);
        }
    }
}

impl Context {
    pub fn new_vertex_buffer<T: VertexData>(&self) -> VertexBuffer<T> {
        VertexBuffer {
            attrib_binder: None,
            buffer: self.gen_buffer(),
            count: 0,
            phantom: PhantomData
        }
    }
}



pub struct IndexBuffer<T: IndexDatum> {
    buffer: Buffer,
    count: usize,
    phantom: PhantomData<*const T>
}

impl<T: IndexDatum> IndexBuffer<T> {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

pub struct IndexBufferBinding<'a, T: IndexDatum + 'a> {
    gl_buffer: ElementArrayBufferBinding<'a>,
    ibo: &'a mut IndexBuffer<T>
}

impl<'a, T: IndexDatum + 'a> IndexBufferBinding<'a, T> {
    pub fn new(gl_buffer: ElementArrayBufferBinding<'a>,
               ibo: &'a mut IndexBuffer<T>)
        -> Self
    {
        IndexBufferBinding {
            gl_buffer: gl_buffer,
            ibo: ibo
        }
    }

    pub fn buffer_data(&mut self, data: &[T], usage: super::BufferDataUsage)
        where [T]: IndexData
    {
        self.ibo.count = data.len();
        self.gl_buffer.buffer_bytes(data.index_bytes(), usage);
    }
}

impl Context {
    pub fn new_index_buffer<T: IndexDatum>(&self) -> IndexBuffer<T> {
        IndexBuffer {
            buffer: self.gen_buffer(),
            count: 0,
            phantom: PhantomData
        }
    }
}



#[macro_export]
macro_rules! attrib_pointers {
    ($gl:expr, $vbo:expr, {
        $($field_name:ident => $field_attrib:expr),*
    }) => {
        {
            let mut binder = $crate::AttribBinder::new();
            $(binder.add(stringify!($field_name), $field_attrib).unwrap());*;
            binder
        }
    }
}

#[macro_export]
macro_rules! bind_attrib_pointers {
    ($gl:expr, $vbo:expr, {
        $($field_name:ident => $field_attrib:expr),*
    }) => {
        {
            let vbo = $vbo;
            let binder = {
                attrib_pointers!($gl, vbo, {
                    $($field_name => $field_attrib),*
                })
            };
            vbo.bind_attrib_pointers(binder);
        }
    }
}

#[macro_export]
macro_rules! bind_vertex_buffer {
    ($gl:expr, $vbo:expr) => {
        {
            let vbo = $vbo;

            let gl_buffer = bind_array_buffer!($gl, vbo.buffer_mut());
            vbo.bind(&gl_buffer).unwrap();
            $crate::VertexBufferBinding::new(gl_buffer, vbo)
        }
    }
}

#[macro_export]
macro_rules! bind_index_buffer {
    ($gl:expr, $ibo:expr) => {
        {
            let ibo = $ibo;

            let gl_buffer = bind_element_array_buffer!($gl, ibo.buffer_mut());
            $crate::IndexBufferBinding::new(gl_buffer, ibo)
        }
    }
}
