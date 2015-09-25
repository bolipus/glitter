use std::marker::PhantomData;
use std::collections::hash_map::{HashMap, Entry};
use gl;
use gl::types::*;
use context::Context;
use renderbuffer::{Renderbuffer, RenderbufferTarget};
use texture::{Texture, TextureType, ImageTargetType,
              Texture2d, Tx2dImageTarget};
use types::{BufferBits, GLError, GLFramebufferError};

pub struct Framebuffer {
    gl_id: GLuint
}

impl Framebuffer {
    pub unsafe fn from_gl(id: GLuint) -> Self {
        Framebuffer { gl_id: id }
    }

    pub fn gl_id(&self) -> GLuint {
        self.gl_id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.gl_id as *const GLuint);
        }
    }
}

enum BuilderAttachment<'a> {
    Texture2d(&'a mut Texture2d, i32),
    Renderbuffer(&'a mut Renderbuffer)
}

pub struct FramebufferBuilder<'a> {
    gl: &'a mut Context,
    attachments: HashMap<FramebufferAttachment, BuilderAttachment<'a>>
}

impl<'a> FramebufferBuilder<'a> {
    fn new(gl: &'a mut Context) -> Self {
        FramebufferBuilder {
            gl: gl,
            attachments: HashMap::new()
        }
    }

    pub fn texture_2d(mut self,
                      attachment: FramebufferAttachment,
                      texture: &'a mut Texture2d,
                      level: i32)
        -> Self
    {
        let attached = BuilderAttachment::Texture2d(texture, level);
        match self.attachments.entry(attachment) {
            Entry::Occupied(mut e) => { e.insert(attached); },
            Entry::Vacant(e) => { e.insert(attached); }
        };

        self
    }

    pub fn renderbuffer(mut self,
                        attachment: FramebufferAttachment,
                        renderbuffer: &'a mut Renderbuffer)
        -> Self
    {
        let attached = BuilderAttachment::Renderbuffer(renderbuffer);
        match self.attachments.entry(attachment) {
            Entry::Occupied(mut e) => { e.insert(attached); },
            Entry::Vacant(e) => { e.insert(attached); }
        };

        self
    }

    pub fn try_unwrap(self) -> Result<Framebuffer, GLError> {
        let mut fbo = unsafe { self.gl.gen_framebuffer() };

        // TODO: Use `bind_framebuffer!` macro here
        let mut gl_fbo = self.gl.framebuffer.bind(&mut fbo);

        for (attachment, attached) in self.attachments.into_iter() {
            match attached {
                BuilderAttachment::Texture2d(texture, level) => {
                    gl_fbo.texture_2d(attachment,
                                      Tx2dImageTarget::Texture2d,
                                      texture,
                                      level);
                },
                BuilderAttachment::Renderbuffer(renderbuffer) => {
                    gl_fbo.renderbuffer(attachment, renderbuffer);
                }
            }
        }

        match gl_fbo.check_framebuffer_status() {
            Some(err) => { Err(err.into()) },
            None => { Ok(fbo) }
        }
    }

    pub fn unwrap(self) -> Framebuffer {
        self.try_unwrap().unwrap()
    }
}

impl Context {
    pub unsafe fn gen_framebuffer(&self) -> Framebuffer {
        let mut id : GLuint = 0;

        gl::GenFramebuffers(1, &mut id as *mut GLuint);
        dbg_gl_sanity_check! {
            GLError::InvalidValue => "`n` is negative",
            _ => "Unknown error"
        }

        Framebuffer {
            gl_id: id
        }
    }
}



gl_enum! {
    pub gl_enum FramebufferTarget {
        Framebuffer as FRAMEBUFFER = gl::FRAMEBUFFER
    }
}

gl_enum! {
    pub gl_enum FramebufferAttachment {
        ColorAttachment0 as COLOR_ATTACHMENT0 = gl::COLOR_ATTACHMENT0,
        DepthAttachment as DEPTH_ATTACHMENT = gl::DEPTH_ATTACHMENT,
        StencilAttachment as STENCIL_ATTACHMENT = gl::STENCIL_ATTACHMENT
    }
}

pub struct FramebufferBinding<'a> {
    phantom: PhantomData<&'a mut Framebuffer>
}

impl<'a> FramebufferBinding<'a> {
    fn target(&self) -> FramebufferTarget {
        FramebufferTarget::Framebuffer
    }

    pub fn check_framebuffer_status(&self) -> Option<GLFramebufferError> {
        unsafe {
            match gl::CheckFramebufferStatus(self.target().gl_enum()) {
                gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
                    Some(GLFramebufferError::IncompleteAttachment)
                },
                // gl::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => {
                //     Some(GLFramebufferError::IncompleteDimensions)
                // },
                gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                    Some(GLFramebufferError::IncompleteMissingAttachment)
                },
                gl::FRAMEBUFFER_UNSUPPORTED => {
                    Some(GLFramebufferError::Unsupported)
                },
                _ => { None }
            }
        }
    }

    pub fn renderbuffer(&mut self,
                        attachment: FramebufferAttachment,
                        renderbuffer: &mut Renderbuffer)
    {
        let renderbuffer_target = RenderbufferTarget::Renderbuffer;
        unsafe {
            gl::FramebufferRenderbuffer(self.target().gl_enum(),
                                        attachment.gl_enum(),
                                        renderbuffer_target.gl_enum(),
                                        renderbuffer.gl_id());
            dbg_gl_sanity_check! {
                GLError::InvalidEnum => "`target` is not `GL_FRAMEBUFFER`, `attachment` is not a valid attachment point, or `renderbuffer` is not `GL_RENDERBUFFER` and `renderbuffer` is not 0",
                GLError::InvalidOperation => "Framebuffer 0 is bound, or `renderbuffer` is neither 0 nor the name of an existing renderbuffer object",
                _ => "Unknown error"
            }
        }
    }

    pub fn texture_2d<T, I>(&mut self,
                            attachment: FramebufferAttachment,
                            tex_target: I,
                            texture: &mut Texture<T>,
                            level: i32)
        where T: TextureType, I: Into<T::ImageTargetType>
    {
        debug_assert!(level == 0);

        unsafe {
            gl::FramebufferTexture2D(self.target().gl_enum(),
                                     attachment.gl_enum(),
                                     tex_target.into().gl_enum(),
                                     texture.gl_id(),
                                     level as GLint);
            dbg_gl_sanity_check! {
                GLError::InvalidEnum => "`target` is not `GL_FRAMEBUFFER`, `attachment` is not an accepted attachment point, or `textarget` is not an accepted texture target and texture is not 0",
                GLError::InvalidValue => "`level` is not 0 and `texture` is not 0",
                GLError::InvalidOperation => "Framebuffer object 0 is bound, `texture` is neither 0 nor the name of an existing texture object, or `textarget` is not a valid target for `texture`",
                _ => "Unknown error"
            }
        }
    }

    pub fn clear(&mut self, buffers: BufferBits) {
        unsafe {
            gl::Clear(buffers.bits());
            dbg_gl_sanity_check! {
                GLError::InvalidValue => "`mask` includes a bit other than an allowed value",
                _ => "Unkown error"
            }
        }
    }
}

pub struct FramebufferBinder;
impl FramebufferBinder {
    pub unsafe fn current_binding(&mut self) -> FramebufferBinding {
        FramebufferBinding { phantom: PhantomData }
    }

    pub fn bind(&mut self, fbo: &mut Framebuffer) -> FramebufferBinding {
        let binding = FramebufferBinding { phantom: PhantomData };
        unsafe {
            gl::BindFramebuffer(binding.target().gl_enum(), fbo.gl_id());
            dbg_gl_sanity_check! {
                GLError::InvalidEnum => "`target` is not `GL_FRAMEBUFFER`",
                _ => "Unknown error"
            }
        }
        binding
    }
}
