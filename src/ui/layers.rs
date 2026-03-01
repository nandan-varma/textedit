//! Layer management system for organizing UI rendering by z-index
//!
//! Layers provide:
//! - Logical grouping of primitives (background, content, overlays)
//! - Dirty tracking to minimize re-rendering
//! - Z-index based ordering for correct draw order

use crate::ui::primitives::{Primitive, RenderList};

/// Named layers for different parts of the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerId {
    /// Background elements (editor bg, gutter bg)
    Background,
    /// Selection highlights
    Selection,
    /// Match highlights from find
    MatchHighlights,
    /// Main text content
    Text,
    /// Line numbers in gutter
    LineNumbers,
    /// Cursor
    Cursor,
    /// Status bar
    StatusBar,
    /// Scrollbars
    Scrollbar,
    /// Modal overlay (darkened background)
    ModalOverlay,
    /// Modal content
    Modal,
}

impl LayerId {
    /// Returns the base z-index for this layer
    pub fn z_index(&self) -> i32 {
        match self {
            LayerId::Background => 0,
            LayerId::Selection => 15,
            LayerId::MatchHighlights => 18,
            LayerId::Text => 20,
            LayerId::LineNumbers => 25,
            LayerId::Cursor => 30,
            LayerId::StatusBar => 40,
            LayerId::Scrollbar => 50,
            LayerId::ModalOverlay => 100,
            LayerId::Modal => 110,
        }
    }

    /// Returns all layer IDs in z-index order
    pub fn all() -> &'static [LayerId] {
        &[
            LayerId::Background,
            LayerId::Selection,
            LayerId::MatchHighlights,
            LayerId::Text,
            LayerId::LineNumbers,
            LayerId::Cursor,
            LayerId::StatusBar,
            LayerId::Scrollbar,
            LayerId::ModalOverlay,
            LayerId::Modal,
        ]
    }
}

/// A single layer containing primitives
#[derive(Debug)]
pub struct Layer {
    id: LayerId,
    primitives: RenderList,
    dirty: bool,
    visible: bool,
}

impl Layer {
    pub fn new(id: LayerId) -> Self {
        Self {
            id,
            primitives: RenderList::new(),
            dirty: true,
            visible: true,
        }
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    pub fn z_index(&self) -> i32 {
        self.id.z_index()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.dirty = true;
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Clear and replace primitives, marking dirty
    pub fn set_primitives(&mut self, primitives: RenderList) {
        self.primitives = primitives;
        self.dirty = true;
    }

    /// Clear primitives
    pub fn clear(&mut self) {
        if !self.primitives.is_empty() {
            self.primitives.clear();
            self.dirty = true;
        }
    }

    /// Get primitives for rendering (only if visible)
    pub fn primitives(&self) -> Option<&RenderList> {
        if self.visible {
            Some(&self.primitives)
        } else {
            None
        }
    }
}

/// Manages all layers and provides unified rendering
pub struct LayerManager {
    layers: Vec<Layer>,
    /// Cached combined render list (updated when any layer is dirty)
    combined: RenderList,
    combined_dirty: bool,
}

impl LayerManager {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(LayerId::all().len());
        for id in LayerId::all() {
            layers.push(Layer::new(*id));
        }

        Self {
            layers,
            combined: RenderList::new(),
            combined_dirty: true,
        }
    }

    /// Get a mutable reference to a specific layer
    pub fn layer_mut(&mut self, id: LayerId) -> &mut Layer {
        self.combined_dirty = true;
        &mut self.layers[Self::layer_index(id)]
    }

    /// Get an immutable reference to a specific layer
    pub fn layer(&self, id: LayerId) -> &Layer {
        &self.layers[Self::layer_index(id)]
    }

    fn layer_index(id: LayerId) -> usize {
        match id {
            LayerId::Background => 0,
            LayerId::Selection => 1,
            LayerId::MatchHighlights => 2,
            LayerId::Text => 3,
            LayerId::LineNumbers => 4,
            LayerId::Cursor => 5,
            LayerId::StatusBar => 6,
            LayerId::Scrollbar => 7,
            LayerId::ModalOverlay => 8,
            LayerId::Modal => 9,
        }
    }

    /// Set primitives for a layer
    pub fn set_layer(&mut self, id: LayerId, primitives: RenderList) {
        self.layer_mut(id).set_primitives(primitives);
    }

    /// Clear a specific layer
    pub fn clear_layer(&mut self, id: LayerId) {
        self.layer_mut(id).clear();
    }

    /// Clear all layers
    pub fn clear_all(&mut self) {
        for layer in &mut self.layers {
            layer.clear();
        }
        self.combined_dirty = true;
    }

    /// Check if any layer is dirty
    pub fn any_dirty(&self) -> bool {
        self.combined_dirty || self.layers.iter().any(|l| l.is_dirty())
    }

    /// Mark all layers as clean
    pub fn mark_all_clean(&mut self) {
        for layer in &mut self.layers {
            layer.mark_clean();
        }
        self.combined_dirty = false;
    }

    /// Get the combined render list from all visible layers, sorted by z-index
    pub fn render_list(&mut self) -> &RenderList {
        if self.any_dirty() {
            self.rebuild_combined();
        }
        &self.combined
    }

    fn rebuild_combined(&mut self) {
        self.combined.clear();

        for layer in &self.layers {
            if let Some(primitives) = layer.primitives() {
                for primitive in primitives.iter() {
                    self.combined.push(primitive.clone());
                }
            }
        }

        // Sort by z-index for correct draw order
        self.combined.sort_by_z_index();

        // Mark all clean
        for layer in &mut self.layers {
            layer.mark_clean();
        }
        self.combined_dirty = false;
    }

    /// Get separate lists for color and text primitives (useful for two-pipeline rendering)
    pub fn partition(&mut self) -> (Vec<&Primitive>, Vec<&Primitive>) {
        if self.any_dirty() {
            self.rebuild_combined();
        }
        self.combined.partition()
    }

    /// Show/hide modal layers
    pub fn set_modal_visible(&mut self, visible: bool) {
        self.layer_mut(LayerId::ModalOverlay).set_visible(visible);
        self.layer_mut(LayerId::Modal).set_visible(visible);
    }

    /// Check if modal is visible
    pub fn is_modal_visible(&self) -> bool {
        self.layer(LayerId::Modal).is_visible()
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::primitives::{Point, Rect};

    #[test]
    fn test_layer_ids_ordered() {
        let ids = LayerId::all();
        for i in 1..ids.len() {
            assert!(
                ids[i].z_index() >= ids[i - 1].z_index(),
                "LayerId::all() should be in z-index order"
            );
        }
    }

    #[test]
    fn test_layer_manager_new() {
        let manager = LayerManager::new();
        assert_eq!(manager.layers.len(), LayerId::all().len());
        assert!(manager.any_dirty()); // New manager has dirty layers
    }

    #[test]
    fn test_layer_dirty_tracking() {
        let mut layer = Layer::new(LayerId::Text);
        assert!(layer.is_dirty()); // New layer is dirty

        layer.mark_clean();
        assert!(!layer.is_dirty());

        let mut list = RenderList::new();
        list.push(Primitive::rect(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            [1.0, 0.0, 0.0, 1.0],
            0,
        ));
        layer.set_primitives(list);
        assert!(layer.is_dirty()); // Setting primitives marks dirty
    }

    #[test]
    fn test_layer_visibility() {
        let mut layer = Layer::new(LayerId::Modal);
        assert!(layer.is_visible());
        assert!(layer.primitives().is_some());

        layer.set_visible(false);
        assert!(!layer.is_visible());
        assert!(layer.primitives().is_none());
    }

    #[test]
    fn test_combined_render_list() {
        let mut manager = LayerManager::new();

        let mut bg_list = RenderList::new();
        bg_list.push(Primitive::rect(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            [0.1, 0.1, 0.1, 1.0],
            0,
        ));
        manager.set_layer(LayerId::Background, bg_list);

        let mut text_list = RenderList::new();
        text_list.push(Primitive::text(
            "Hello",
            Point::new(10.0, 10.0),
            [1.0, 1.0, 1.0, 1.0],
            20,
        ));
        manager.set_layer(LayerId::Text, text_list);

        let combined = manager.render_list();
        assert_eq!(combined.len(), 2);

        // Background should come first (lower z-index)
        let primitives: Vec<_> = combined.iter().collect();
        assert!(primitives[0].z_index() <= primitives[1].z_index());
    }

    #[test]
    fn test_modal_visibility() {
        let mut manager = LayerManager::new();

        manager.set_modal_visible(false);
        assert!(!manager.is_modal_visible());
        assert!(!manager.layer(LayerId::ModalOverlay).is_visible());

        manager.set_modal_visible(true);
        assert!(manager.is_modal_visible());
    }
}
