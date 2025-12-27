use std::collections::HashMap;

/// Texture resource for materials
#[derive(Debug, Clone)]
pub struct Texture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub mipmaps: Vec<Vec<u8>>, // Optional mipmap data
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
    Depth32Float,
}

impl Texture {
    /// Create a new texture from raw data
    pub fn new(name: &str, width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            data,
            format,
            mipmaps: Vec::new(),
        }
    }

    /// Create a solid color texture
    pub fn solid_color(name: &str, color: [u8; 4]) -> Self {
        let mut data = Vec::with_capacity(4);
        data.extend_from_slice(&color);

        Self::new(name, 1, 1, data, TextureFormat::Rgba8)
    }

    /// Create a checkerboard texture for debugging
    pub fn checkerboard(name: &str, width: u32, height: u32, color1: [u8; 4], color2: [u8; 4]) -> Self {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let checker = ((x / 8) + (y / 8)) % 2 == 0;
                let color = if checker { color1 } else { color2 };
                data.extend_from_slice(&color);
            }
        }

        Self::new(name, width, height, data, TextureFormat::Rgba8)
    }

    /// Get the size in bytes of one pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self.format {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Rgb8 => 3,
            TextureFormat::R8 => 1,
            TextureFormat::Depth32Float => 4,
        }
    }

    /// Get the total size in bytes
    pub fn size_in_bytes(&self) -> usize {
        self.data.len()
    }

    /// Check if texture has valid dimensions
    pub fn is_valid(&self) -> bool {
        let expected_size = (self.width * self.height * self.bytes_per_pixel()) as usize;
        self.data.len() == expected_size
    }
}

/// Texture manager for handling texture resources
pub struct TextureManager {
    pub textures: HashMap<String, Texture>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Register a texture
    pub fn register_texture(&mut self, texture: Texture) {
        self.textures.insert(texture.name.clone(), texture);
    }

    /// Get a texture by name
    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }

    /// Create and register default textures
    pub fn create_default_textures(&mut self) {
        // Placeholder texture (white)
        let placeholder = Texture::solid_color("placeholder", [255, 255, 255, 255]);
        self.register_texture(placeholder);

        // Checkerboard for debugging
        let checkerboard = Texture::checkerboard(
            "checkerboard",
            64,
            64,
            [255, 255, 255, 255], // White
            [0, 0, 0, 255],       // Black
        );
        self.register_texture(checkerboard);
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sampler state for texture sampling
#[derive(Debug, Clone)]
pub struct SamplerState {
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mipmap_filter: MipmapMode,
    pub address_u: AddressMode,
    pub address_v: AddressMode,
    pub address_w: AddressMode,
    pub anisotropy: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MipmapMode {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

impl Default for SamplerState {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: MipmapMode::Linear,
            address_u: AddressMode::Repeat,
            address_v: AddressMode::Repeat,
            address_w: AddressMode::Repeat,
            anisotropy: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_creation() {
        let data = vec![255, 0, 0, 255]; // Red pixel
        let texture = Texture::new("red", 1, 1, data, TextureFormat::Rgba8);

        assert_eq!(texture.name, "red");
        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.bytes_per_pixel(), 4);
        assert!(texture.is_valid());
    }

    #[test]
    fn test_solid_color_texture() {
        let texture = Texture::solid_color("white", [255, 255, 255, 255]);

        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.data, vec![255, 255, 255, 255]);
        assert!(texture.is_valid());
    }

    #[test]
    fn test_checkerboard_texture() {
        let texture = Texture::checkerboard("check", 8, 8, [255, 0, 0, 255], [0, 255, 0, 255]);

        assert_eq!(texture.width, 8);
        assert_eq!(texture.height, 8);
        assert_eq!(texture.size_in_bytes(), 256); // 8 * 8 * 4
        assert!(texture.is_valid());
    }

    #[test]
    fn test_texture_manager() {
        let mut manager = TextureManager::new();

        let texture = Texture::solid_color("test", [128, 128, 128, 255]);
        manager.register_texture(texture);

        assert!(manager.get_texture("test").is_some());
        assert!(manager.get_texture("nonexistent").is_none());
    }

    #[test]
    fn test_sampler_state_default() {
        let sampler = SamplerState::default();

        assert_eq!(sampler.min_filter, FilterMode::Linear);
        assert_eq!(sampler.mag_filter, FilterMode::Linear);
        assert_eq!(sampler.address_u, AddressMode::Repeat);
        assert_eq!(sampler.anisotropy, 1.0);
    }
}
