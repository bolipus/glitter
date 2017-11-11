initSidebarItems({"constant":[["CLAMP_TO_EDGE","Wrap a texture by clamping it within the range `[1/2x, 1 - 1/2x]`, where `x` is the dimension of the texture being clamped."],["LINEAR","When texturing a pixel, select the four texels that are closest to the center of the pixel, and compute the result by taking a weighted average of each texel."],["LINEAR_MIPMAP_LINEAR","When texturing a pixel, select the two mipmaps that are nearest in size to the pixel. For each, select the four texels that are closest to the center of the pixel, and compute the weighted average. Finally, take the resulting two weighted averages of the texels, and take the weighted average of both based on the mipmaps."],["LINEAR_MIPMAP_NEAREST","When texturing a pixel, select the mipmap that is nearest in size to the pixel, select the four texels that are closest to the center of the pixel, and compute the result by taking the weighted average of each texel."],["MIRRORED_REPEAT","Wrap a texture by repeating it front-to-back, then back-to-front, then repeating."],["NEAREST","When texturing a pixel, select the texel that is closest to the center of the pixel."],["NEAREST_MIPMAP_LINEAR","When texturing a pixel, select the two mipmaps that are nearest in size to the pixel, select the texel in each that is closest to the center of the pixel, and compute the result by taking the weighted average of each texel."],["NEAREST_MIPMAP_NEAREST","When texturing a pixel, select the mipmap that is nearest in size to the pixel, and select the texel that is closest to the center of the pixel."],["REPEAT","Wrap a texture by repeating it over and over again."],["TEXTURE_2D","This constant is designed to be used in glitter wherever the constant `GL_TEXTURE_2D` is used in plain OpenGL code."],["TEXTURE_CUBE_MAP","This constant is designed to be used in glitter wherever the constant `GL_TEXTURE_CUBE_MAP` is used in plain OpenGL code."],["TEXTURE_CUBE_MAP_NEGATIVE_X","The negative-X image target face of a cubemap."],["TEXTURE_CUBE_MAP_NEGATIVE_Y","The negative-Y image target face of a cubemap."],["TEXTURE_CUBE_MAP_NEGATIVE_Z","The negative-Z image target face of a cubemap."],["TEXTURE_CUBE_MAP_POSITIVE_X","The positive-X image target face of a cubemap."],["TEXTURE_CUBE_MAP_POSITIVE_Y","The positive-Y image target face of a cubemap."],["TEXTURE_CUBE_MAP_POSITIVE_Z","The positive-Z image target face of a cubemap."]],"enum":[["TextureBindingTarget","Represents all of the possible types of OpenGL textures."],["TextureFilter","Represents the different forms of texture filtering, which determines how a texture will be sampled when drawn."],["TextureMipmapFilter","Represents the different forms of texture filtering when using mipmaps."],["TextureWrapMode","The wrapping modes when drawing a texture."],["Tx2dImageTarget","The possible image targets for `GL_TEXTURE_2D` (only one variant, since this is the 2D texture)."],["TxCubeMapImageTarget","The possible 2D image targets for a cubemap texture."]],"struct":[["Texture","A type of OpenGL texture."],["Tx2d","The `TextureType` for 2-dimensional textures."],["TxCubeMap","The `TextureType` for cubemap textures."],["VariantTexture2d","This is a unit type that is used to be coerced into select enum variants."]],"trait":[["ImageTargetType","A trait implemented for types that are used to represent all of the possible 2D images that make up a specific implementation of `TextureType`. For more details, read the `TextureType` documentation."],["TextureType","A trait implemented for a type that represent a type of texture (such as 2D textures or cube map textures).  For example, [`TxCubeMap`] (struct.TxCubeMap.html) is a type that implements `TextureType`, and it represents cube map textures."]],"type":[["Texture2d","An OpenGL texture with 2-dimensional image data."],["TextureCubeMap","An OpenGL texture used to hold a cubemap texture, made up of 6 2-dimensional images (one for each face of a cube)."]]});