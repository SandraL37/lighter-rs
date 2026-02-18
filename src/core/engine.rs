use crate::core::{
    dirty::DirtyFlags,
    error::*,
    layout::{AvailableSpace, Size},
    node::{Node, NodeKey},
    render::Renderer,
    tree::Tree,
};

pub struct Engine<R: Renderer> {
    tree: Tree,
    renderer: R,
    size: Size<u32>,
}

impl<R: Renderer> Engine<R> {
    pub fn new(tree: Tree, renderer: R) -> Self {
        let size = renderer.get_size();
        Engine {
            tree,
            renderer,
            size,
        }
    }

    pub fn resize(&mut self, size: Size<u32>) -> Result<()> {
        self.renderer.resize(size)?;
        self.size = size;
        Ok(())
    }

    pub fn frame(&mut self) -> Result<()> {
        self.tree.compute_layout(Size::wh(
            AvailableSpace::Definite(self.size.width as f32),
            AvailableSpace::Definite(self.size.height as f32),
        ));

        let commands = self.tree.build_render_list();

        self.renderer.render(&commands)?;

        Ok(())
    }

    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn get_node(&self, key: NodeKey) -> Result<&Node> {
        self.tree.get_node_by_key(key)
    }

    pub fn mutate(
        &mut self,
        key: NodeKey,
        flags: DirtyFlags,
        f: impl FnOnce(&mut Node),
    ) -> Result<()> {
        self.tree.mutate_by_key(key, flags, f)
    }
}
