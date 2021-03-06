//! Contains all of the OpenGL state types related to shader programs.

use std::ptr;
use std::error;
use std::fmt;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::ffi::CString;
use gl;
use gl::types::*;
use types::{GLObject, GLError};
use context::{AContext, BaseContext, ContextOf};
use program::{Program, ProgramAttrib, ProgramUniform};
use shader::Shader;
use uniform_data::{UniformData, UniformDatumType, UniformPrimitiveType};

unsafe fn _get_program_iv(program: &Program,
                          pname: GLenum,
                          params: *mut GLint)
{
    gl::GetProgramiv(program.id(), pname, params);
    dbg_gl_sanity_check! {
        GLError::InvalidEnum => "`pname` is not an accepted value",
        GLError::InvalidValue => "`program` is not a value generated by OpenGL",
        GLError::InvalidOperation => "`program` does not refer to a program object",
        _ => "Unknown error"
    }
}

/// Provides a safe interface for creating program objects. A
/// `ProgramBuilder` can be created using the [`gl.build_program`]
/// (trait.ContextProgramBuilderExt.html#method.build_program) method.
pub struct ProgramBuilder<'a, C>
    where C: AContext + 'a
{
    gl: &'a C,
    shaders: &'a [Shader]
}

impl<'a, C> ProgramBuilder<'a, C>
    where C: AContext
{
    /// Create a new program builder.
    pub fn new(gl: &'a C, shaders: &'a [Shader])
        -> Self
    {
        ProgramBuilder { gl: gl, shaders: shaders }
    }

    /// Create and link the program object with the provided shaders, or
    /// return an error.
    ///
    /// # Failures
    /// An error will be returned if there was an error linking the program
    /// object.
    ///
    /// # Panics
    /// This function will panic if an OpenGL
    /// error was generated with debug assertions enabled.
    pub fn try_unwrap(self) -> Result<Program, GLError> {
        unsafe {
            let mut program = try! {
                self.gl.create_program().or_else(|_| {
                    let msg = "Error creating OpenGL program";
                    Err(GLError::Message(msg.to_owned()))
                })
            };

            for shader in self.shaders {
                self.gl.attach_shader(&mut program, shader);
            }

            try!(self.gl.link_program(&mut program));
            Ok(program)
        }
    }

    /// Create and link the program object with the provided shaders,
    /// or panic.
    ///
    /// # Panics
    /// This function will panic if there was an error linking
    /// the program object or if an OpenGL error was generated with debug
    /// assertions enabled.
    pub fn unwrap(self) -> Program {
        self.try_unwrap().unwrap()
    }
}

/// The extension trait that adds the `build_program` method.
pub trait ContextProgramBuilderExt: AContext + Sized {
    /// Create a new program builder, providing a safe interface
    /// for constructing a program object. See the [`ProgramBuilder`]
    /// (struct.ProgramBuilder.html) docs for more details.
    fn build_program<'a>(&'a self, shaders: &'a [Shader])
        -> ProgramBuilder<'a, Self>
    {
        ProgramBuilder::new(self, shaders)
    }
}

impl<C: AContext> ContextProgramBuilderExt for C {

}

/// An extension trait that includes program-related OpenGL methods.
pub trait ContextProgramExt: BaseContext {
    /// Create a new program object that has no shaders attached, or return
    /// an error if a shader object could not be created.
    ///
    /// # Safety
    /// Most OpenGL function calls assume a program object will have
    /// already had shaders attached and linked using [`gl.attach_shader`]
    /// (trait.ContextProgramExt.html#method.attach_shader) and
    /// [`gl.link_program`](trait.ContextProgramExt.html#method.link_program),
    /// respectively. Violating this invariant is considered undefined
    /// behavior in glitter.
    ///
    /// # See also
    /// [`glCreateProgram`](http://docs.gl/es2/glCreateProgram) OpenGL docs
    ///
    /// [`gl.build_program`](trait.ContextProgramBuilderExt.html#method.build_program):
    /// A safe wrapper for creating a program object.
    unsafe fn create_program(&self) -> Result<Program, ()> {
        let id = gl::CreateProgram();
        if id > 0 {
            Ok(Program::from_raw(id))
        }
        else {
            Err(())
        }
    }

    /// Attach a shader to a program object.
    ///
    /// # Panics
    /// This function will panic if the provided shader is already attached
    /// to the program object.
    ///
    /// # See also
    /// [`glAttachShader`](http://docs.gl/es2/glAttachShader) OpenGL docs
    fn attach_shader(&self, program: &mut Program, shader: &Shader) {
        unsafe {
            gl::AttachShader(program.id(), shader.id());
            dbg_gl_error! {
                GLError::InvalidValue => "One of either `program` or `shader` is not an OpenGL object",
                GLError::InvalidOperation => "`shader` is already attached to `program`, `shader` is not a shader object, or `program` is not a program object",
                _ => "Unknown error"
            }
        }
    }

    /// Link the program object, so that it can be used for rendering. Returns
    /// an error if the program could not be linked.
    ///
    /// # Failures
    /// If the `GL_LINK_STATUS` after linking the program was not `GL_TRUE`,
    /// then an error object containing the program's info log will
    /// be returned. Refer to the [`glLinkProgram`]
    /// (http://docs.gl/es2/glLinkProgram) OpenGL docs for the possible
    /// causes of failure.
    ///
    /// # Panics
    /// This function will panic if an OpenGL error is generated and debug
    /// assertions are enabled.
    ///
    /// # See also
    /// [`glLinkProgram`](http://docs.gl/es2/glLinkProgram) OpenGL docs
    fn link_program(&self, program: &mut Program) -> Result<(), GLError> {
        let success = unsafe {
            gl::LinkProgram(program.id());
            dbg_gl_error! {
                GLError::InvalidValue => "`program` is not a value from OpenGL",
                GLError::InvalidOperation => "`program` is not a program object",
                _ => "Unknown error"
            }

            let mut link_status : GLint = 0;
            _get_program_iv(program,
                            gl::LINK_STATUS,
                            &mut link_status as *mut GLint);

            link_status == gl::TRUE as GLint
        };

        if success {
            Ok(())
        }
        else {
            let msg = match self.get_program_info_log(&program) {
                Some(s) => { s },
                None => { String::from("[Unknown program error]") }
            };
            Err(GLError::Message(msg))
        }
    }

    /// Return the information log for the program object, if any is
    /// available.
    ///
    /// # Note
    /// If the info log returned by the OpenGL driver contained an invalid
    /// UTF-8 sequence, `None` will be returned.
    ///
    /// # See also
    /// [`glGetProgramInfoLog`](http://docs.gl/es2/glGetProgramInfoLog) OpenGL docs
    fn get_program_info_log(&self, program: &Program) -> Option<String> {
        unsafe {
            let mut info_length : GLint = 0;
            _get_program_iv(program,
                            gl::INFO_LOG_LENGTH,
                            &mut info_length as *mut GLint);

            if info_length > 0 {
                let mut bytes = Vec::<u8>::with_capacity(info_length as usize);

                gl::GetProgramInfoLog(program.id(),
                                      info_length,
                                      ptr::null_mut(),
                                      bytes.as_mut_ptr() as *mut GLchar);
                dbg_gl_sanity_check! {
                    GLError::InvalidValue => "`program` is not a value generated by OpenGL, or `maxLength` < 0",
                    GLError::InvalidOperation => "`program` is not a program object",
                    _ => "Unknown error"
                }
                bytes.set_len((info_length - 1) as usize);

                String::from_utf8(bytes).ok()
            }
            else {
                None
            }
        }
    }

    /// Retrieve a program attribute's index by name, or return an error
    /// if the attribute was not found in the program.
    ///
    /// # Panics
    /// This function will panic if an OpenGL error was generated and
    /// debug assertions are enabled.
    ///
    /// # See also
    /// [`glGetAttribLocation`](http://docs.gl/es2/glGetAttribLocation) OpenGL
    /// docs
    fn get_attrib_location<'a>(&self, program: &Program, name: &'a str)
        -> Result<ProgramAttrib, UnknownProgramAttrib<'a>>
    {
        let err = Err(UnknownProgramAttrib { name: name });

        let c_str = match CString::new(name) {
            Ok(s) => { s },
            Err(_) => { return err }
        };

        let str_ptr = c_str.as_ptr() as *const GLchar;
        unsafe {
            let index = gl::GetAttribLocation(program.id(), str_ptr);
            dbg_gl_error! {
                GLError::InvalidOperation => "`program` has not been linked, `program` is not a program object, or `program` is not a value generated by OpenGL",
                _ => "Unknown error"
            }

            if index >= 0 {
                Ok(ProgramAttrib { gl_index: index as GLuint })
            }
            else {
                err
            }
        }
    }

    /// Retrieve a program uniform's index by name, or return an error
    /// if the uniform was not found within the program.
    ///
    /// # Panics
    /// This function will panic if an OpenGL error was generated and
    /// debug assertions are enabled.
    ///
    /// # See also
    /// [`glGetUniformLocation`](http://docs.gl/es2/glGetUniformLocation)
    /// OpenGL docs
    fn get_uniform_location<'a>(&self, program: &Program, name: &'a str)
        -> Result<ProgramUniform, UnknownProgramUniform<'a>>
    {
        let err = Err(UnknownProgramUniform { name: name });

        let c_str = match CString::new(name) {
            Ok(s) => { s },
            Err(_) => { return err; }
        };

        let str_ptr = c_str.as_ptr() as *const GLchar;
        unsafe {
            let index = gl::GetUniformLocation(program.id(), str_ptr);
            dbg_gl_error! {
                GLError::InvalidValue => "`program` is not a value generated by OpenGL",
                GLError::InvalidOperation => "`program` is not a program object, or has not been successfully linked",
                _ => "Unknown error"
            }

            if index >= 0 {
                Ok(ProgramUniform { gl_index: index as GLuint })
            }
            else {
                err
            }
        }
    }

    /// Set the value of a uniform variable within the provided program
    /// object binding.
    ///
    /// - `_gl_program`: The program binding to change.
    /// - `uniform`: The location of the uniform variable. This value
    ///              can be retrieved using [`gl.get_uniform_location`]
    ///              (trait.ContextProgramExt.html#method.get_uniform_location)
    ///              method.
    /// - `val`: The value to set the uniform variable to. See the
    ///          [`UniformData`](../../uniform_data/trait.UniformData.html)
    ///          docs for more details about the types of uniform data.
    ///
    /// # Panics
    /// This function will panic if an OpenGL error is generated and
    /// debug assertions are enabled.
    ///
    /// # See also
    /// [`glUniform`](http://docs.gl/es2/glUniform) OpenGL docs
    fn set_uniform<T>(&self,
                      _gl_program: &ProgramBinding,
                      uniform: ProgramUniform,
                      val: T)
        where T: UniformData
    {
        let idx = uniform.gl_index as GLint;
        let count = val.uniform_elements() as GLsizei;
        let ptr = val.uniform_bytes().as_ptr();
        unsafe {
            match T::uniform_datum_type() {
                UniformDatumType::Vec1(p) => {
                    match p {
                        UniformPrimitiveType::Float => {
                            gl::Uniform1fv(idx, count, ptr as *const GLfloat);
                        },
                        UniformPrimitiveType::Int => {
                            gl::Uniform1iv(idx, count, ptr as *const GLint);
                        }
                    }
                },
                UniformDatumType::Vec2(p) => {
                    match p {
                        UniformPrimitiveType::Float => {
                            gl::Uniform2fv(idx, count, ptr as *const GLfloat);
                        },
                        UniformPrimitiveType::Int => {
                            gl::Uniform2iv(idx, count, ptr as *const GLint);
                        }
                    }
                },
                UniformDatumType::Vec3(p) => {
                    match p {
                        UniformPrimitiveType::Float => {
                            gl::Uniform3fv(idx, count, ptr as *const GLfloat);
                        },
                        UniformPrimitiveType::Int => {
                            gl::Uniform3iv(idx, count, ptr as *const GLint);
                        }
                    }
                },
                UniformDatumType::Vec4(p) => {
                    match p {
                        UniformPrimitiveType::Float => {
                            gl::Uniform4fv(idx, count, ptr as *const GLfloat);
                        },
                        UniformPrimitiveType::Int => {
                            gl::Uniform4iv(idx, count, ptr as *const GLint);
                        }
                    }
                },
                UniformDatumType::Matrix2x2 => {
                    gl::UniformMatrix2fv(idx,
                                         count,
                                         gl::FALSE,
                                         ptr as *const GLfloat);
                },
                UniformDatumType::Matrix3x3 => {
                    gl::UniformMatrix3fv(idx,
                                         count,
                                         gl::FALSE,
                                         ptr as *const GLfloat);
                },
                UniformDatumType::Matrix4x4 => {
                    gl::UniformMatrix4fv(idx,
                                         count,
                                         gl::FALSE,
                                         ptr as *const GLfloat);
                },
            }

            dbg_gl_error! {
                GLError::InvalidOperation => "Invalid uniform operation",
                GLError::InvalidValue => "`count` < 0 or `transpose` is not GL_FALSE",
                _ => "Unknown error"
            }
        }
    }
}

impl<C: BaseContext> ContextProgramExt for C {

}



/// An OpenGL context that has a free program binding.
pub trait ProgramContext: AContext {
    /// The type of binder this context contains.
    type Binder: BorrowMut<ProgramBinder>;

    /// The OpenGL context that will be returned after binding a program.
    type Rest: AContext;

    /// Split the context into a binder and the remaining context.
    fn split_program(self) -> (Self::Binder, Self::Rest);

    /// Bind a program to this context's program, returning a new
    /// context and a binding.
    fn use_program<'a>(self, program: &'a mut Program)
        -> (ProgramBinding<'a>, Self::Rest)
        where Self: Sized
    {
        let (mut binder, rest) = self.split_program();
        (binder.borrow_mut().bind(program), rest)
    }
}

impl<B, F, P, R, T> ProgramContext for ContextOf<B, F, P, R, T>
    where P: BorrowMut<ProgramBinder>
{
    type Binder = P;
    type Rest = ContextOf<B, F, (), R, T>;

    fn split_program(self) -> (Self::Binder, Self::Rest) {
        self.swap_program(())
    }
}

impl<'a, B, F, P, R, T> ProgramContext for &'a mut ContextOf<B, F, P, R, T>
    where &'a mut P: BorrowMut<ProgramBinder>
{
    type Binder = &'a mut P;
    type Rest = ContextOf<&'a mut B, &'a mut F, (), &'a mut R, &'a mut T>;

    fn split_program(self) -> (Self::Binder, Self::Rest) {
        let gl = self.borrowed_mut();
        gl.swap_program(())
    }
}



/// Represents a program that has been bound to the context.
pub struct ProgramBinding<'a> {
    _phantom_ref: PhantomData<&'a mut Program>,
    _phantom_ptr: PhantomData<*mut ()>
}

/// The OpenGL state representing the active program target.
pub struct ProgramBinder {
    _phantom: PhantomData<*mut ()>
}

impl ProgramBinder {
    /// Get the current program binder.
    pub unsafe fn current() -> Self {
        ProgramBinder {
            _phantom: PhantomData
        }
    }

    /// Bind a program to the context, returning a binding.
    pub fn bind<'a>(&mut self, program: &'a mut Program) -> ProgramBinding<'a>
    {
        let binding = ProgramBinding {
            _phantom_ref: PhantomData,
            _phantom_ptr: PhantomData
        };
        unsafe {
            gl::UseProgram(program.id());
            dbg_gl_error! {
                GLError::InvalidValue => "`program` is neither 0 nor an object generated by OpenGL",
                GLError::InvalidOperation => "`program` is not a program object or `program` could not be made part of the current state",
                _ => "Unknown error"
            }
        }
        binding
    }
}



/// An error that represents a program attribute that could not be found.
#[derive(Debug)]
pub struct UnknownProgramAttrib<'a> {
    name: &'a str
}

impl<'a> fmt::Display for UnknownProgramAttrib<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown program attribute: {:?}", self.name)
    }
}

impl<'a> error::Error for UnknownProgramAttrib<'a> {
    fn description(&self) -> &str {
        "The desired program attribute was not found"
    }
}



/// An error that represents a program uniform that could not be found.
#[derive(Debug)]
pub struct UnknownProgramUniform<'a> {
    name: &'a str
}

impl<'a> fmt::Display for UnknownProgramUniform<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown program uniform: {:?}", self.name)
    }
}

impl<'a> error::Error for UnknownProgramUniform<'a> {
    fn description(&self) -> &str {
        "The desired program uniform was not found"
    }
}
