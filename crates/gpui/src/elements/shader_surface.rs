use crate::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Style, StyleRefinement, Styled, Window,
};
use refineable::Refineable as _;

/// A layout element intended for renderer-side shader surfaces.
///
/// This element itself does not issue draw calls. It reports its resolved bounds
/// during prepaint so renderer integrations can enqueue custom shader work.
pub struct ShaderSurface {
    on_prepaint: Option<Box<dyn FnOnce(Bounds<Pixels>, &mut Window, &mut App)>>,
    style: StyleRefinement,
}

/// Construct a shader-surface element and run `on_prepaint` with its resolved bounds.
pub fn shader_surface(
    on_prepaint: impl 'static + FnOnce(Bounds<Pixels>, &mut Window, &mut App),
) -> ShaderSurface {
    ShaderSurface {
        on_prepaint: Some(Box::new(on_prepaint)),
        style: StyleRefinement::default(),
    }
}

impl IntoElement for ShaderSurface {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ShaderSurface {
    type RequestLayoutState = Style;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style.clone(), [], cx);
        (layout_id, style)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if let Some(on_prepaint) = self.on_prepaint.take() {
            on_prepaint(bounds, window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _style: &mut Style,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }
}

impl Styled for ShaderSurface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
