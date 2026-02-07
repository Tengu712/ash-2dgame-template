use ash::{Device, vk};

pub struct InfoBase<T> {
    pub info: Vec<T>,
    pub set: vk::DescriptorSet,
    pub binding: u32,
    pub count: u32,
    pub ty: vk::DescriptorType,
    pub offset: u32,
}

impl<T> InfoBase<T> {
    fn build(&self) -> vk::WriteDescriptorSet<'_> {
        vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(self.binding)
            .descriptor_count(self.count)
            .descriptor_type(self.ty)
            .dst_array_element(self.offset)
    }
}

pub enum Info {
    Buffer(InfoBase<vk::DescriptorBufferInfo>),
    ImageView(InfoBase<vk::DescriptorImageInfo>),
    Sampler(InfoBase<vk::DescriptorImageInfo>),
}

impl Info {
    fn build(&self) -> vk::WriteDescriptorSet<'_> {
        match self {
            Self::Buffer(info) => info.build().buffer_info(&info.info),
            Self::ImageView(info) => info.build().image_info(&info.info),
            Self::Sampler(info) => info.build().image_info(&info.info),
        }
    }
}

pub fn update_descriptor_sets(device: &Device, infos: &[Info]) {
    let writes = infos.iter().map(|info| info.build()).collect::<Vec<_>>();
    unsafe { device.update_descriptor_sets(&writes, &[]) };
}

macro_rules! define_info_from {
    ($name:ident, $ty:ident, $value:ident => $info:expr,) => {
        impl Info {
            pub fn $name(
                values: &[vk::$ty],
                set: vk::DescriptorSet,
                binding: &vk::DescriptorSetLayoutBinding,
                offset: u32,
            ) -> Self {
                Self::$ty(InfoBase {
                    info: values.iter().map(|$value| $info).collect(),
                    set,
                    binding: binding.binding,
                    count: binding.descriptor_count,
                    ty: binding.descriptor_type,
                    offset,
                })
            }
        }
    };
}

define_info_from!(
    from_buffer,
    Buffer,
    value => vk::DescriptorBufferInfo {
        buffer: *value,
        offset: 0,
        range: vk::WHOLE_SIZE,
    },
);
define_info_from!(
    from_image_view,
    ImageView,
    value => vk::DescriptorImageInfo {
        sampler: vk::Sampler::null(),
        image_view: *value,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    },
);
define_info_from!(
    from_sampler,
    Sampler,
    value => vk::DescriptorImageInfo {
        sampler: *value,
        image_view: vk::ImageView::null(),
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    },
);
